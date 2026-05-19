# This code is part of Qiskit.
#
# (C) Copyright 2026 IBM. All Rights Reserved.
#
# This code is licensed under the Apache License, Version 2.0. You may
# obtain a copy of this license in the LICENSE.txt file in the root directory
# of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
#
# Any modifications or derivative works of this code must retain this
# copyright notice, and modified files need to carry a notice indicating
# that they have been altered from the originals.
"""Qiskit backend provider for QRMI IQM backends."""

# pylint: disable=import-error

from __future__ import annotations

import dataclasses
import time
from collections import Counter
from collections.abc import Callable
from datetime import date
from typing import Any
from uuid import UUID
import warnings
import json

import numpy as np

from qiskit import QuantumCircuit
from qiskit.providers import JobStatus, JobV1, Options
from qiskit.result import Counts, Result

from qrmi import QuantumResource, Payload, TaskStatus

from iqm.iqm_client import CircuitCompilationOptions, CircuitValidationError
from iqm.iqm_client.util import to_json_dict
from iqm.pulse import Circuit
from iqm.qiskit_iqm.iqm_backend import IQMBackendBase
from iqm.qiskit_iqm.qiskit_to_iqm import MeasurementKey, serialize_instructions
from iqm.station_control.client.qon import ObservationFinder
from iqm.iqm_server_client.models import CalibrationSet, QualityMetricSet
from iqm.station_control.interface.models import (
    CircuitBatch,
    CircuitMeasurementResultsBatch,
    DynamicQuantumArchitecture,
    RunRequest,
)
from .service import QRMIService

# ---------------------------------------------------------------------------
# Job
# ---------------------------------------------------------------------------

STATUS_MAP = {
    TaskStatus.Queued: JobStatus.QUEUED,
    TaskStatus.Running: JobStatus.RUNNING,
    TaskStatus.Completed: JobStatus.DONE,
    TaskStatus.Failed: JobStatus.ERROR,
    TaskStatus.Cancelled: JobStatus.CANCELLED,
}


class IQMJobCustom(JobV1):
    """Qiskit job backed by IQMClientProtocol instead of IQMClient/CircuitJob."""

    # How long to wait between status-poll requests (seconds).
    POLL_INTERVAL: float = 2.0

    def __init__(
        self,
        backend: QRMIBackend,
        job_id: UUID,
        circuits: CircuitBatch,
        shots: int,
    ):
        super().__init__(backend, job_id=str(job_id))
        self._job_id = job_id
        self._circuits = circuits
        self._shots = shots
        self._cached_result: list[tuple[str, list[str], Counts]] | None = None
        self.circuit_metadata: list | None = None

    # ------------------------------------------------------------------
    # JobV1 interface
    # ------------------------------------------------------------------

    def submit(self) -> None:
        raise NotImplementedError(
            "Job is submitted automatically in QRMIBackend.run()."
        )

    def status(self) -> JobStatus:
        iqm_status = self.backend().qrmi.task_status(self._job_id)
        return STATUS_MAP.get(iqm_status)

    def result(
        self, *, timeout: float = 300.0, poll_interval: float = POLL_INTERVAL
    ) -> Result:
        """Poll until the job finishes, then return the Qiskit Result."""
        deadline = time.monotonic() + timeout
        while True:
            iqm_status = self.backend().qrmi.task_status(self._job_id)
            if iqm_status not in [TaskStatus.Running, TaskStatus.Queued]:
                break
            if time.monotonic() > deadline:
                raise TimeoutError(
                    f"Job {self._job_id} did not finish within {timeout} seconds."
                )
            time.sleep(poll_interval)

        result_dict: dict[str, Any] = {
            "backend_name": self.backend().name,
            "backend_version": "",
            "qobj_id": "",
            "job_id": str(self._job_id),
            "success": False,
            "date": date.today().isoformat(),
            "results": [],
        }

        if iqm_status == TaskStatus.Completed:
            if self._cached_result is None:
                result = self.backend().qrmi.task_result(self._job_id).value
                result_json = json.loads(result)
                measurements = result_json["measurements"]
                self._cached_result = _format_results(
                    measurements, self._circuits, self._shots
                )

            result_dict["success"] = True
            result_dict["results"] = [
                {
                    "shots": len(meas),
                    "success": True,
                    "data": {
                        "memory": meas,
                        "counts": counts,
                        "metadata": (
                            self.circuit_metadata[i]
                            if self.circuit_metadata is not None
                            else {}
                        ),
                    },
                    "header": {"name": name},
                }
                for i, (name, meas, counts) in enumerate(self._cached_result)
            ]

        return Result.from_dict(result_dict)

    def cancel(self) -> bool:
        warnings.warn("cancel() is not supported by IQMJobCustom.")
        return False


# ---------------------------------------------------------------------------
# Backend
# ---------------------------------------------------------------------------


class QRMIBackend(IQMBackendBase):
    """IQMBackend that uses IQMClientProtocol instead of IQMClient.

    Args:
        client: Object satisfying IQMClientProtocol.  Its methods map 1-to-1
                onto the REST API endpoints used by the original IQMBackend.
        calibration_set_id: ID of the calibration set to use.  None means the
                server default (queried on every run to detect calset changes).
        use_metrics: If True, fetch calibration quality metrics from the server.
        kwargs: Forwarded to IQMBackendBase.
    """

    def __init__(
        self,
        qrmi: QuantumResource,
        *,
        calibration_set_id: str | UUID | None = None,
        use_metrics: bool = False,
        **kwargs,
    ):
        if calibration_set_id is not None and not isinstance(calibration_set_id, UUID):
            calibration_set_id = UUID(calibration_set_id)

        self.target_json = json.loads(qrmi.target().value)

        self._use_default_calibration_set = calibration_set_id is None
        architecture = DynamicQuantumArchitecture.model_validate(
            self.target_json["dynamic_quantum_architecture"]
        )

        cal_set = CalibrationSet.model_validate(self.target_json["calibration_set"])
        qms = QualityMetricSet.model_validate(self.target_json["quality_metrics"])

        metrics = (
            ObservationFinder(cal_set.observations + qms.observations)
            if use_metrics
            else None
        )

        super().__init__(architecture, metrics=metrics, **kwargs)
        self.qrmi: QuantumResource = qrmi
        self._calibration_set_id: UUID = architecture.calibration_set_id
        self._max_circuits: int | None = None

    # ------------------------------------------------------------------
    # BackendV2 interface
    # ------------------------------------------------------------------

    @classmethod
    def _default_options(cls) -> Options:
        return Options()

    @property
    def max_circuits(self) -> int | None:
        """max_circuits property"""
        return self._max_circuits

    @max_circuits.setter
    def max_circuits(self, value: int | None) -> None:
        self._max_circuits = value

    # ------------------------------------------------------------------
    # Public API
    # ------------------------------------------------------------------

    def run(
        self,
        run_input: QuantumCircuit | list[QuantumCircuit],
        *,
        use_timeslot: bool = False,
        **options,
    ) -> IQMJobCustom:
        """Transpile and submit circuits; return a job object."""
        run_request = self.create_run_request(run_input, **options)
        iqmjson = run_request.model_dump_json()

        payload = Payload.IQMServer(
            iqmjson=iqmjson,
            job_type="circuit",
            use_timeslot=use_timeslot,
            tag=None,
        )

        job_id = self.qrmi.task_start(payload)
        job = IQMJobCustom(self, job_id, run_request.circuits, run_request.shots)
        job.circuit_metadata = [
            c.metadata if isinstance(c, Circuit) else {} for c in run_request.circuits
        ]
        return job

    def create_run_request(
        self,
        run_input: QuantumCircuit | list[QuantumCircuit],
        shots: int = 1024,
        circuit_compilation_options: CircuitCompilationOptions | None = None,
        circuit_callback: Callable[[list[QuantumCircuit]], Any] | None = None,
        qubit_index_to_name: dict[int, str] | None = None,
        **unknown_options,
    ) -> RunRequest:
        """Build a RunRequest without submitting it.

        Identical logic to the original IQMBackend.create_run_request().
        """
        circuits = [run_input] if isinstance(run_input, QuantumCircuit) else run_input
        if not circuits:
            raise ValueError("Empty list of circuits submitted for execution.")

        # Handle deprecated options
        if (
            "max_circuit_duration_over_t2" in unknown_options
            or "heralding_mode" in unknown_options
        ):
            warnings.warn(
                DeprecationWarning(
                    "max_circuit_duration_over_t2 and heralding_mode are deprecated; "
                    "use circuit_compilation_options instead."
                )
            )
        if circuit_compilation_options is None:
            cc_kwargs = {}
            if "max_circuit_duration_over_t2" in unknown_options:
                cc_kwargs["max_circuit_duration_over_t2"] = unknown_options.pop(
                    "max_circuit_duration_over_t2"
                )
            if "heralding_mode" in unknown_options:
                cc_kwargs["heralding_mode"] = unknown_options.pop("heralding_mode")
            circuit_compilation_options = CircuitCompilationOptions(**cc_kwargs)

        if unknown_options:
            warnings.warn(f"Unknown backend option(s): {unknown_options}")

        if circuit_callback:
            circuit_callback(circuits)

        if qubit_index_to_name is None:
            qubit_index_to_name = self._idx_to_qb

        circuits_serialized: CircuitBatch = [
            self._serialize_circuit(circuit, qubit_index_to_name)
            for circuit in circuits
        ]

        # Detect default calibration set changes (same logic as original IQMBackend)
        if self._use_default_calibration_set:
            default_dqa = DynamicQuantumArchitecture.model_validate(
                self.target_json["dynamic_quantum_architecture"]
            )
            if self._calibration_set_id != default_dqa.calibration_set_id:
                warnings.warn(
                    f"Server default calibration set has changed from {self._calibration_set_id} "
                    f"to {default_dqa.calibration_set_id}. "
                    "Create a new QRMIBackend if you wish to transpile using the new calibration set."
                )

        try:
            run_request = _build_run_request(
                circuits_serialized,
                calibration_set_id=self._calibration_set_id,
                shots=shots,
                options=circuit_compilation_options,
            )
        except CircuitValidationError as cve:
            raise CircuitValidationError(
                f"{cve}\nMake sure circuits were transpiled with this backend."
            ) from cve

        return run_request

    def serialize_circuit(
        self, circuit: QuantumCircuit, qubit_index_to_name: dict[int, str] | None = None
    ) -> Circuit:
        """Public alias kept for API compatibility with IQMBackend."""
        return self._serialize_circuit(circuit, qubit_index_to_name or self._idx_to_qb)

    # ------------------------------------------------------------------
    # Internal helpers
    # ------------------------------------------------------------------

    def _serialize_circuit(
        self, circuit: QuantumCircuit, qubit_index_to_name: dict[int, str]
    ) -> Circuit:
        instructions = tuple(
            serialize_instructions(circuit, qubit_index_to_name=qubit_index_to_name)
        )
        try:
            metadata = to_json_dict(circuit.metadata)
        except ValueError:
            warnings.warn(
                f"Metadata of circuit {circuit.name} was dropped because it could not be serialized to JSON."
            )
            metadata = None
        return Circuit(name=circuit.name, instructions=instructions, metadata=metadata)


# ---------------------------------------------------------------------------
# Private helpers
# ---------------------------------------------------------------------------


def _build_run_request(
    circuits: CircuitBatch,
    *,
    calibration_set_id: UUID,
    shots: int,
    options: CircuitCompilationOptions,
) -> RunRequest:
    """Construct a RunRequest from serialized circuits and options."""
    # options_dict = options.model_dump(exclude_none=True)
    options_dict = {
        k: v for k, v in dataclasses.asdict(options).items() if v is not None
    }
    return RunRequest(
        circuits=circuits,
        calibration_set_id=calibration_set_id,
        shots=shots,
        **options_dict,
    )


def _format_results(
    measurements: CircuitMeasurementResultsBatch,
    circuits: CircuitBatch,
    requested_shots: int,
) -> list[tuple[str, list[str], Counts]]:
    """Convert raw measurement results to Qiskit format.

    Replicates IQMJob._iqm_format_results() / _iqm_format_measurement_results().
    """
    results = []
    for i, (circuit, meas_result) in enumerate(zip(circuits, measurements)):
        name = circuit.name if isinstance(circuit, Circuit) else f"circuit_{i}"
        bitstrings = _format_single_circuit_results(meas_result, requested_shots)
        results.append((name, bitstrings, Counts(Counter(bitstrings))))
    return results


def _format_single_circuit_results(
    measurement_results: dict, requested_shots: int
) -> list[str]:
    """Convert one circuit's measurement dict to a list of bitstrings."""
    formatted: dict[int, np.ndarray] = {}
    for k, v in measurement_results.items():
        mk = MeasurementKey.from_string(k)
        res = np.array(v, dtype=int)
        if res.shape[1] != 1:
            raise ValueError(f"Unexpected result shape {res.shape} for key {mk}")
        res = res[:, 0]
        creg = formatted.setdefault(
            mk.creg_idx, np.zeros((len(res), mk.creg_len), dtype=int)
        )
        creg[:, mk.clbit_idx] = res

    shots = requested_shots
    return [
        " ".join("".join(map(str, formatted[idx][s, :])) for idx in sorted(formatted))[
            ::-1
        ]
        for s in range(shots)
    ]


class IQMProvider:
    """Provider for IQM backends.

    IQMProvider connects to a quantum computer through an IQM Server.
    If the server requires user authentication, you can provide it either using environment
    variables, or as keyword arguments to IQMProvider. The user authentication kwargs are passed
    through to :class:`~iqm.iqm_client.iqm_client.IQMClient` as is, and are documented there.

    Args:
        url: URL of the IQM Server (e.g. "https://resonance.iqm.tech/").
        quantum_computer: ID or alias of the quantum computer to connect to, if the IQM Server
            instance controls more than one (e.g. "garnet"). ``None`` means connect to the
            default one.

    """

    def __init__(
        self,
        **args,
    ):
        pass

    def get_backend(
        self,
        name: str | None = None,
        calibration_set_id: UUID | None = None,
        *,
        use_metrics: bool = False,
    ) -> QRMIBackend:
        """IQMBackend instance associated with this provider.

        Args:
            name: Optional name of a facade backend to request, see :class:`IQMFacadeBackend`.
            calibration_set_id: ID of the calibration set to be used with the backend.
                Affects both the transpilation target and the circuit execution.
                If None, the server default calibration set will be used.
            use_metrics: If True, the backend will provide calibration data and related quality metrics
                to the transpilation target to improve the transpilation. The default value is set to False
                until quality metrics become available on the Resonance API.

        Returns:
            Backend instance for connecting to a quantum computer.

        """
        service = QRMIService()
        if name is not None:
            qrmi = service.resource(name.replace(":", "_"))
        else:
            qrmi = service.resources()[0]
        return QRMIBackend(
            qrmi, calibration_set_id=calibration_set_id, use_metrics=use_metrics
        )
