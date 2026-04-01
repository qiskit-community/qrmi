# This code is part of Qiskit.
#
# (C) Copyright 2025 Pasqal, IBM. All Rights Reserved.
#
# This code is licensed under the Apache License, Version 2.0. You may
# obtain a copy of this license in the LICENSE.txt file in the root directory
# of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
#
# Any modifications or derivative works of this code must retain this
# copyright notice, and modified files need to carry a notice indicating
# that they have been altered from the originals.

"""Sampler V2 base class for Pasqal QRMI resources."""

import json
import time
from dataclasses import dataclass, field
from typing import Any

from qiskit import QuantumCircuit
from qiskit.primitives import BasePrimitiveJob
from qiskit.providers import JobStatus
from qiskit.providers.jobstatus import JOB_FINAL_STATES
from qiskit_pasqal_provider.providers import SamplerV2 as PasqalSamplerV2
from qiskit_pasqal_provider.providers.pulse_utils import (
    gen_seq,
    get_register_from_circuit,
)
from qiskit_pasqal_provider.providers.result import PasqalResult

from qrmi import Payload, QuantumResource, TaskStatus

from .target import get_device

STATUS_MAP = {
    TaskStatus.Queued: JobStatus.QUEUED,
    TaskStatus.Running: JobStatus.RUNNING,
    TaskStatus.Completed: JobStatus.DONE,
    TaskStatus.Failed: JobStatus.ERROR,
    TaskStatus.Cancelled: JobStatus.CANCELLED,
}


@dataclass
class Options:
    """Options for :class:`~.QRMIPasqalBackend`."""

    default_shots: int = 1024
    run_options: dict[str, Any] = field(default_factory=dict)


class QRMIPasqalJob(BasePrimitiveJob[PasqalResult, JobStatus]):
    """Representation of a QRMI Pasqal sampler job."""

    def __init__(
        self,
        qrmi: QuantumResource,
        job_id: str,
        backend_name: str,
        *,
        poll_interval_seconds: float = 1.0,
        timeout_seconds: float | None = None,
        delete_job: bool = False,
    ) -> None:
        super().__init__(job_id)
        self._qrmi = qrmi
        self._backend_name = backend_name
        self._poll_interval_seconds = poll_interval_seconds
        self._timeout_seconds = timeout_seconds
        self._delete_job = delete_job
        self._last_status = None
        self._result = None

    def __del__(self) -> None:
        if self._delete_job is True:
            self._qrmi.task_stop(self._job_id)

    def cancel(self) -> None:
        """Cancel the job."""
        self._qrmi.task_stop(self._job_id)

    def result(self) -> PasqalResult:
        """Return the result of the job."""
        if self._result is not None and self._last_status == JobStatus.DONE:
            return self._result

        start_time = time.time()
        while True:
            status = self.status()
            if status in JOB_FINAL_STATES:
                break
            if self._timeout_seconds is not None and (
                time.time() - start_time
            ) > self._timeout_seconds:
                raise TimeoutError(
                    f"QRMI task {self._job_id} timed out after {self._timeout_seconds}s."
                )
            time.sleep(self._poll_interval_seconds)

        if self._last_status != JobStatus.DONE:
            raise RuntimeError(f"QRMI task {self._job_id} ended with status {self._last_status}.")

        raw_result = self._qrmi.task_result(self._job_id).value
        parsed_result = json.loads(raw_result) if isinstance(raw_result, str) else raw_result
        self._result = PasqalResult(
            backend_name=self._backend_name,
            job_id=self._job_id,
            results=parsed_result,
            metadata={"status": "DONE", "success": True},
        )
        return self._result

    def status(self) -> JobStatus:
        """Return the status of the job."""
        if self._last_status is None or self._last_status not in JOB_FINAL_STATES:
            self._last_status = STATUS_MAP[self._qrmi.task_status(self._job_id)]
        return self._last_status

    def done(self) -> bool:
        """Return whether the job has successfully run."""
        return self.status() == JobStatus.DONE

    def running(self) -> bool:
        """Return whether the job is actively running."""
        return self.status() == JobStatus.RUNNING

    def cancelled(self) -> bool:
        """Return whether the job has been cancelled."""
        return self.status() == JobStatus.CANCELLED

    def in_final_state(self) -> bool:
        """Return whether the job reached a final state."""
        return self.status() in JOB_FINAL_STATES


class QRMIPasqalBackend:
    """QRMI-backed backend adapter consumed by qiskit-pasqal-provider SamplerV2."""

    def __init__(
        self,
        qrmi: QuantumResource,
        *,
        options: dict | None = None,
    ) -> None:
        self._qrmi = qrmi
        self._options = Options(**options) if options else Options()

    def run(
        self,
        run_input: QuantumCircuit,
        shots: int | None = None,
        values: dict | None = None,
        **_: Any,
    ) -> QRMIPasqalJob:
        analog_register = get_register_from_circuit(run_input)
        device = get_device(self._qrmi)

        seq = gen_seq(
            analog_register=analog_register,
            device=device,
            circuit=run_input,
        )
        if values:
            seq = seq.build(**values)

        payload = Payload.PasqalCloud(
            sequence=seq.to_abstract_repr(),
            job_runs=shots if shots is not None else self._options.default_shots,
        )
        task_id = self._qrmi.task_start(payload)
        run_options = self._options.run_options
        return QRMIPasqalJob(
            qrmi=self._qrmi,
            job_id=task_id,
            backend_name=self._qrmi.resource_id(),
            poll_interval_seconds=float(run_options.get("poll_interval_seconds", 1.0)),
            timeout_seconds=run_options.get("timeout_seconds"),
            delete_job=bool(run_options.get("delete_job", False)),
        )


class QPPSamplerV2(PasqalSamplerV2):
    """SamplerV2 for Pasqal execution through QRMI."""

    def __init__(
        self,
        qrmi: QuantumResource,
        *,
        options: dict | None = None,
    ) -> None:
        super().__init__(backend=QRMIPasqalBackend(qrmi=qrmi, options=options))
