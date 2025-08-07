# -*- coding: utf-8 -*-

# This code is part of Qiskit.
#
# (C) Copyright 2025 IBM. All Rights Reserved.
#
# This code is licensed under the Apache License, Version 2.0. You may
# obtain a copy of this license in the LICENSE.txt file in the root directory
# of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
#
# Any modifications or derivative works of this code must retain this
# copyright notice, and modified files need to carry a notice indicating
# that they have been altered from the originals.

"""Runtime job"""

import time
from typing import Union
from qiskit_ibm_runtime.utils.result_decoder import ResultDecoder
from qiskit.primitives import BasePrimitiveJob, PrimitiveResult
from qiskit.providers import JobStatus
from qiskit.providers.jobstatus import JOB_FINAL_STATES
from qrmi import QuantumResource, TaskStatus

STATUS_MAP = {
    TaskStatus.Queued: JobStatus.QUEUED,
    TaskStatus.Running: JobStatus.RUNNING,
    TaskStatus.Completed: JobStatus.DONE,
    TaskStatus.Failed: JobStatus.ERROR,
    TaskStatus.Cancelled: JobStatus.CANCELLED,
}


class RuntimeJobV2(BasePrimitiveJob[PrimitiveResult, TaskStatus]):
    """Representation of a runtime V2 primitive exeuction."""

    def __init__(
        self, qrmi: QuantumResource, job_id: str, *, delete_job: bool = False
    ) -> None:
        """RuntimeJob constructor.

        Args:
            qrmi: QRMI object.
            job_id: Job ID.
            delete_job: True if you want delete the job in the destructor.
        """
        super().__init__(job_id)
        self._qrmi = qrmi
        self._last_status = None
        self._result = None
        self._delete_job = delete_job

    def __del__(self):
        """RuntimeJob destructor."""
        if self._delete_job is True:
            self._qrmi.task_stop(self._job_id)

    def cancel(self) -> None:
        """Cancel the job."""
        self._qrmi.task_stop(self._job_id)

    def result(self) -> PrimitiveResult:
        """Return the results of the job."""
        if self._last_status is not None and self._result is not None:
            return self._result

        while True:
            if self.in_final_state() is True:
                break

            time.sleep(1)

        result = self._qrmi.task_result(self._job_id)
        self._result = ResultDecoder.decode(result.value)
        return self._result

    def status(self) -> JobStatus:
        """Return the status of the job.

        Returns:
            Status of this job.
        """
        if self._last_status is None or self._last_status not in JOB_FINAL_STATES:
            self._last_status = self._qrmi.task_status(self._job_id)
        return STATUS_MAP.get(self._last_status)

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
        """Return whether the job is in a final job state such as ``DONE`` or ``ERROR``."""
        return self.status() in JOB_FINAL_STATES
