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

"""Executor and client-side SamplerV2 / EstimatorV2 on top of QRMI."""

from __future__ import annotations

import json
import time
from collections.abc import Iterable
from typing import TYPE_CHECKING, Any

from qiskit.primitives import BasePrimitiveJob
from qiskit.primitives.base import BaseEstimatorV2, BaseSamplerV2
from qiskit.primitives.containers.estimator_pub import EstimatorPubLike
from qiskit.primitives.containers.sampler_pub import SamplerPubLike
from qiskit.providers import BackendV2, JobStatus
from qiskit.providers.jobstatus import JOB_FINAL_STATES

# NOTE: these are internal modules of qiskit-ibm-runtime and may change between releases.
from qiskit_ibm_runtime.decoders.executor.decoder import (
    BaseClientSideResultDecoder,
    ExecutorResultDecoder,
)
from qiskit_ibm_runtime.decoders.executor_estimator.decoder import (
    ClientSideEstimatorDecoder,
)
from qiskit_ibm_runtime.decoders.executor_sampler.decoder import (
    ClientSideSamplerResultDecoder,
)
from qiskit_ibm_runtime.executor_estimator.prepare import prepare as estimator_prepare
from qiskit_ibm_runtime.executor_sampler.prepare import prepare
from qiskit_ibm_runtime.options_models.estimator import EstimatorOptions
from qiskit_ibm_runtime.options_models.executor import ExecutorOptions
from qiskit_ibm_runtime.options_models.sampler import SamplerOptions
from qiskit_ibm_runtime.quantum_program import QuantumProgram
from qiskit_ibm_runtime.quantum_program.params_converters import (
    QUANTUM_PROGRAM_PARAMS_CONVERTERS,
)

from qrmi import ( # pylint: disable=no-name-in-module
    Payload,
    QuantumResource,
    TaskStatus,
    QrmiError,
)

if TYPE_CHECKING:
    import samplomatic

SCHEMA_VERSION = "v2.0"  # same as qiskit_ibm_runtime.Executor._SCHEMA_VERSION

_STATUS_MAP = {
    TaskStatus.Queued: JobStatus.QUEUED,
    TaskStatus.Running: JobStatus.RUNNING,
    TaskStatus.Completed: JobStatus.DONE,
    TaskStatus.Failed: JobStatus.ERROR,
    TaskStatus.Cancelled: JobStatus.CANCELLED,
}


def _resolve_mode(mode) -> tuple[QuantumResource, BackendV2 | None]:
    """Return ``(qrmi, backend)`` for ``mode``.

    ``mode`` can be a ``QRMIBackend`` (``qrmi.primitives.ibm.get_backend(qrmi)``), which
    carries its QRMI resource, or a bare QRMI ``QuantumResource`` (then no backend is known,
    so dynamical decoupling cannot be used).
    """
    if isinstance(mode, BackendV2):
        qrmi = getattr(mode, "_qrmi", None)
        if qrmi is None:
            raise TypeError(
                f"{type(mode).__name__} is not backed by QRMI; pass a QRMIBackend "
                "(qrmi.primitives.ibm.get_backend(qrmi)) or a QRMI QuantumResource."
            )
        return qrmi, mode
    if hasattr(mode, "task_start"):  # QRMI QuantumResource
        return mode, None
    raise TypeError(
        f"mode must be a QRMIBackend or a QRMI QuantumResource, got {type(mode)}"
    )


def _backend_for_dd(
    qrmi: QuantumResource, backend: BackendV2 | None, options
) -> BackendV2 | None:
    """Create the QRMIBackend on demand when DD is enabled but only a resource was given."""
    if backend is None and options.dynamical_decoupling.enable:
        from qrmi.primitives.ibm.backend import (
            get_backend,
        )  # fetches target via qrmi.target()

        backend = get_backend(qrmi)
    return backend


def _submit_program(
    qrmi: QuantumResource,
    options: ExecutorOptions,
    program: QuantumProgram | samplomatic.QuantumProgram,
    post_processor: type[BaseClientSideResultDecoder] | None = None,
) -> ExecutorJob:
    """Encode ``program`` as Executor input, submit it through QRMI and return the job.

    ``post_processor`` is attached to the job so that ``result()`` returns a
    ``PrimitiveResult`` (Sampler/Estimator) instead of a ``QuantumProgramResult``.
    """
    if not isinstance(program, QuantumProgram):
        program = QuantumProgram.from_(program)  # accept samplomatic.QuantumProgram
    # Same encoding as qiskit_ibm_runtime.Executor.run (the job `params`)
    params = QUANTUM_PROGRAM_PARAMS_CONVERTERS[SCHEMA_VERSION].encoder(program, options)
    payload = Payload.QiskitPrimitive(
        input=json.dumps(params.model_dump(mode="json")), program_id="executor"
    )
    job_id = qrmi.task_start(payload)
    return ExecutorJob(qrmi, job_id, post_processor=post_processor)


class ExecutorJob(BasePrimitiveJob[Any, JobStatus]):
    """Job handle for an Executor job submitted through QRMI."""

    def __init__(
        self,
        qrmi: QuantumResource,
        job_id: str,
        poll_interval: float = 1.0,
        post_processor: type[BaseClientSideResultDecoder] | None = None,
    ):
        super().__init__(job_id)
        self._qrmi = qrmi
        self._post_processor = post_processor
        self._poll = poll_interval
        self._status: JobStatus | None = None
        self._result: Any = None

    def status(self) -> JobStatus:
        if self._status not in JOB_FINAL_STATES:
            self._status = _STATUS_MAP.get(self._qrmi.task_status(self._job_id))
        return self._status

    def result(self) -> Any:
        """QuantumProgramResult (plain Executor) or PrimitiveResult (Sampler/Estimator)."""
        if self._result is not None:
            return self._result
        while not self.in_final_state():
            time.sleep(self._poll)
        if self._status != JobStatus.DONE:
            raise RuntimeError(f"Job {self._job_id} ended with status {self._status}")

        raw = self._qrmi.task_result(self._job_id).value
        # raw JSON -> QuantumProgramResult
        result = ExecutorResultDecoder.decode(raw)
        # -> PrimitiveResult, using the post-processor chosen by the Sampler/Estimator that
        # submitted the job (instead of relying on the program's semantic role).
        if self._post_processor is not None:
            result = self._post_processor.decode(result)
        self._result = result
        return self._result

    def __del__(self):
        """destructor."""
        try:
            self._qrmi.task_stop(self._job_id)
        except QrmiError:
            pass

    def cancel(self) -> None:
        self._qrmi.task_stop(self._job_id)

    def done(self) -> bool:
        return self.status() == JobStatus.DONE

    def running(self) -> bool:
        return self.status() == JobStatus.RUNNING

    def cancelled(self) -> bool:
        return self.status() == JobStatus.CANCELLED

    def errored(self) -> bool:
        """Return whether the job has failed."""
        return self.status() == JobStatus.ERROR

    def in_final_state(self) -> bool:
        return self.status() in JOB_FINAL_STATES

    def logs(self) -> str:
        """Return the job logs from the QRMI resource.

        Returns:
            The log text of the job.
        """
        return self._qrmi.task_logs(self._job_id)


class ExecutorSampler(BaseSamplerV2):
    """SamplerV2 that runs via the IBM ``executor`` program through QRMI.

    Args:
        mode: A ``QRMIBackend`` (``qrmi.primitives.ibm.get_backend(qrmi)``), or a QRMI
            ``QuantumResource``. With a bare resource, the backend (needed only for dynamical
            decoupling timing) is created on first use via ``get_backend(qrmi)``.
        options: ``SamplerOptions`` or a dict for it (same options as
            ``qiskit_ibm_runtime.executor_sampler.Sampler``).
    """

    def __init__(
        self,
        mode: BackendV2 | QuantumResource,
        *,
        options: SamplerOptions | dict | None = None,
    ):
        super().__init__()
        self._qrmi, self._backend = _resolve_mode(mode)
        if options is None:
            options = SamplerOptions()
        elif isinstance(options, dict):
            options = SamplerOptions(**options)
        self.options = options

    def run(
        self, pubs: Iterable[SamplerPubLike], *, shots: int | None = None
    ) -> ExecutorJob:
        # 1) Client-side pre-processing (identical to executor_sampler.Sampler.run)
        self._backend = _backend_for_dd(self._qrmi, self._backend, self.options)
        program, executor_options = prepare(
            pubs, self.options, shots, add_tags=False, backend=self._backend
        )
        # 2)+3) Encode and submit through QRMI; post-process results as Sampler results
        return _submit_program(
            self._qrmi,
            executor_options,
            program,
            post_processor=ClientSideSamplerResultDecoder,
        )


class ExecutorEstimator(BaseEstimatorV2):
    """EstimatorV2 that runs via the IBM ``executor`` program through QRMI.

    Pre-processing (twirling, ZNE gate folding, PEC/PEA noise injection, TREX, DD) and
    post-processing (expectation values, ZNE extrapolation, ...) are done client-side
    by qiskit-ibm-runtime's ``executor_estimator``.

    Args:
        mode: A ``QRMIBackend`` (``qrmi.primitives.ibm.get_backend(qrmi)``), or a QRMI
            ``QuantumResource``. With a bare resource, the backend (needed only for dynamical
            decoupling timing) is created on first use via ``get_backend(qrmi)``.
        options: ``EstimatorOptions`` or a dict for it.
    """

    def __init__(
        self,
        mode: BackendV2 | QuantumResource,
        *,
        options: EstimatorOptions | dict | None = None,
    ):
        super().__init__()
        self._qrmi, self._backend = _resolve_mode(mode)
        if options is None:
            options = EstimatorOptions()
        elif isinstance(options, dict):
            options = EstimatorOptions(**options)
        self.options = options

    def run(
        self, pubs: Iterable[EstimatorPubLike], *, precision: float | None = None
    ) -> ExecutorJob:
        self._backend = _backend_for_dd(self._qrmi, self._backend, self.options)
        program, executor_options = estimator_prepare(
            pubs, self.options, precision, add_tags=False, backend=self._backend
        )
        return _submit_program(
            self._qrmi,
            executor_options,
            program,
            post_processor=ClientSideEstimatorDecoder,
        )


class Executor:
    """Counterpart of ``qiskit_ibm_runtime.Executor`` that submits through QRMI.

    ``run()`` takes a ``QuantumProgram`` (qiskit-ibm-runtime's or samplomatic's) and returns a
    job whose ``result()`` is a ``QuantumProgramResult`` (raw per-item arrays such as ``meas``).
    The Sampler/Estimator wrappers additionally attach their post-processor to the job, so their
    jobs return a ``PrimitiveResult``.

    Args:
        mode: A ``QRMIBackend`` (``qrmi.primitives.ibm.get_backend(qrmi)``) or a QRMI
            ``QuantumResource``.
        options: ``ExecutorOptions`` or a dict for it. Only ``execution`` and ``experimental``
            are sent (same as ``Executor``); job-level settings such as ``max_execution_time``
            are governed by the QRMI resource configuration.
    """

    def __init__(
        self,
        mode: BackendV2 | QuantumResource,
        *,
        options: ExecutorOptions | dict | None = None,
    ):
        self._qrmi, _ = _resolve_mode(mode)
        if options is None:
            options = ExecutorOptions()
        elif isinstance(options, dict):
            options = ExecutorOptions(**options)
        self.options = options

    def run(self, program: QuantumProgram | samplomatic.QuantumProgram) -> ExecutorJob:
        """Encode a quantum program as Executor input and submit it through QRMI.

        Args:
            program: A ``qiskit_ibm_runtime.quantum_program.QuantumProgram`` or a
                ``samplomatic.QuantumProgram``.

        Returns:
            A job whose ``result()`` is a ``QuantumProgramResult``.
        """
        return _submit_program(self._qrmi, self.options, program)
