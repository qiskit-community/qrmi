# This code is part of Qiskit.
#
# (C) Copyright 2025-2026 Pasqal. All Rights Reserved.
#
# This code is licensed under the Apache License, Version 2.0. You may
# obtain a copy of this license in the LICENSE.txt file in the root directory
# of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
#
# Any modifications or derivative works of this code must retain this
# copyright notice, and modified files need to carry a notice indicating
# that they have been altered from the originals.
"""Pulser remote connection backed by QRMI calls."""

from __future__ import annotations

import json
from json import JSONDecodeError
import logging
import time
from typing import Any, Mapping, Sequence

import pulser
from pulser.abstract_repr import deserialize_device
from pulser.exceptions.serialization import DeserializeDeviceError
from pulser.backend.remote import (
    BatchStatus,
    JobParams,
    JobStatus,
    RemoteConnection,
    RemoteResults,
    RemoteResultsError,
)
from pulser.devices import Device
from pulser.backend.results import Results
from pulser.result import SampledResult

from qrmi import Payload, QuantumResource, TaskStatus, ResourceType  # type: ignore
from qrmi.pulser.service import QRMIService

logger = logging.getLogger(__name__)

_QRMI_TASK_STATUS_MAP: dict[TaskStatus, JobStatus] = {
    TaskStatus.Queued: JobStatus.PENDING,
    TaskStatus.Running: JobStatus.RUNNING,
    TaskStatus.Completed: JobStatus.DONE,
    TaskStatus.Failed: JobStatus.ERROR,
    TaskStatus.Cancelled: JobStatus.CANCELED,
}

JOB_EXECUTION_POLLING_INTERVAL_S = 1

_COMPATIBLE_RESOURCE_TYPES: tuple[ResourceType] = (
    ResourceType.PasqalLocal,
    ResourceType.PasqalCloud,
)


def _normalize_json_payload(payload: Any) -> dict[str, Any]:
    """Return payload as a dictionary from JSON text or dict."""
    if isinstance(payload, str):
        normalized = json.loads(payload)
    else:
        raise TypeError("Unsupported payload type. Expected JSON string or dict.")

    if not isinstance(normalized, dict):
        raise TypeError("Invalid payload. Expected a JSON object.")

    return normalized


class PulserQRMIConnection(RemoteConnection):
    """A connection to Pasqal QRMI resources, to submit Sequences to QPUs."""

    def __init__(self, qrmi: QuantumResource | None = None) -> None:
        self._qrmi = qrmi or self._resolve_single_resource()
        self._task_sequences: dict[str, pulser.Sequence] = {}

        _type: ResourceType = self._qrmi.resource_type()
        if _type not in _COMPATIBLE_RESOURCE_TYPES:
            raise ValueError(
                "PulserQRMIConnection can only be used with 'PasqalLocal' or 'PasqalCloud' QRMI resources,"
                f" got: '{_type}'"
            )

    @staticmethod
    def _resolve_single_resource() -> QuantumResource:
        """Return the only accessible QRMI resource in the current job context."""
        resources = QRMIService().resources()
        if len(resources) == 1:
            return resources[0]
        if len(resources) == 0:
            raise RuntimeError(
                "No accessible QRMI resource found. "
                "Specify one explicitly with PulserQRMIConnection(qrmi=...)."
            )
        resource_ids = ", ".join(resource.resource_id() for resource in resources)
        raise ValueError(
            "Multiple QRMI resources are available in this job context: "
            f"{resource_ids}. Specify one explicitly with "
            "PulserQRMIConnection(qrmi=...)."
        )

    def supports_open_batch(self) -> bool:
        """Flag to confirm this class doesn't support open batch creation."""
        return False

    def fetch_available_devices(self) -> dict[str, Device]:
        """Fetches the devices available through this connection."""

        devices = {}
        # Serialized data from the Pasqal QRMI matches the one from
        # the /api/v1/devices/public-specs cloud endpoint.
        devices_str = self._qrmi.target().value
        try:
            data = json.loads(devices_str)
        except JSONDecodeError:
            logger.exception(
                "Failed to deserialize device information: %s", devices_str
            )
            return devices
        for specs in data:
            name = specs["device_type"]
            try:
                dev = deserialize_device(specs["specs"])
            except DeserializeDeviceError:
                logger.exception("Failed to deserialize device: %s", {name})
                continue
            devices[name] = dev
        return devices

    def _fetch_result(
        self, batch_id: str, job_ids: list[str] | None
    ) -> Sequence[Results]:
        """Fetches the results of a completed batch."""
        jobs = self._query_job_progress(batch_id)
        selected_job_ids = list(jobs.keys()) if job_ids is None else job_ids
        results: list[Results] = []
        for job_id in selected_job_ids:
            if job_id not in jobs:
                raise RemoteResultsError(
                    f"Job {job_id!r} does not exist in batch {batch_id!r}."
                )
            status, result = jobs[job_id]
            if status in (JobStatus.PENDING, JobStatus.RUNNING):
                raise RemoteResultsError(
                    f"The results are not yet available, job {job_id} status is {status}."
                )
            if result is None:
                raise RemoteResultsError(f"No results found for job {job_id}.")
            results.append(result)
        return tuple(results)

    def _get_batch_status(self, batch_id: str) -> BatchStatus:
        """Gets the status of a batch from its ID."""
        statuses = [
            self._to_job_status(self._qrmi.task_status(job_id))
            for job_id in self._get_job_ids(batch_id)
        ]
        if not statuses:
            raise RuntimeError("A QRMI task returned without a status")
        if all(status == JobStatus.PENDING for status in statuses):
            return BatchStatus.PENDING
        if all(status == JobStatus.CANCELED for status in statuses):
            return BatchStatus.CANCELED
        if all(status == JobStatus.ERROR for status in statuses):
            return BatchStatus.ERROR
        if all(
            status not in (JobStatus.PENDING, JobStatus.RUNNING) for status in statuses
        ):
            return BatchStatus.DONE
        return BatchStatus.RUNNING

    def _query_job_progress(
        self, batch_id: str
    ) -> Mapping[str, tuple[JobStatus, Results | None]]:
        """Fetches the status and results of all the jobs in a batch.

        Unlike `_fetch_result`, this method does not raise an error if some
        jobs in the batch do not have results.

        It returns a dictionary mapping the job ID to its status and results.
        """
        progress: dict[str, tuple[JobStatus, Results | None]] = {}
        for job_id in self._get_job_ids(batch_id):
            status = self._to_job_status(self._qrmi.task_status(job_id))
            result = None
            if status == JobStatus.DONE:
                try:
                    result = self._task_result_to_results(job_id)
                except (
                    RemoteResultsError,
                    json.JSONDecodeError,
                    KeyError,
                    TypeError,
                    ValueError,
                ):
                    logger.warning(
                        "Failed to parse QRMI result for job %s", job_id, exc_info=True
                    )
            progress[job_id] = (status, result)
        return progress

    def submit(
        self,
        sequence: pulser.Sequence,
        wait: bool = True,
        open: bool = False,  # pylint: disable=redefined-builtin
        batch_id: str | None = None,
        **kwargs: Any,
    ) -> RemoteResults:
        """Submits the sequence for execution on a remote Pasqal backend."""
        if open or batch_id:
            raise NotImplementedError("Open batches are not implemented in QRMI.")
        sequence = self._add_measurement_to_sequence(sequence)
        # Check that Job Params are correctly defined
        job_params: list[JobParams] = pulser.json.utils.make_json_compatible(
            kwargs.get("job_params", [])
        )

        # Create a new batch by submitting to the targeted qpu.
        # If QRMI exposes device specs for the selected resource, enforce a
        # sequence-device match and validate runs against the target max_runs.
        available_devices = self.fetch_available_devices()

        if available_devices:
            target_device = next(
                (
                    device
                    for device in available_devices.values()
                    if sequence.device.name == device.name
                ),
                None,
            )
            if target_device is None:
                raise ValueError(
                    f"The Sequence's device {sequence.device.name} doesn't match the "
                    "name of a device of any available QPU. Select your device among "
                    "fetch_available_devices() and change your Sequence's device using "
                    "its switch_device method."
                )
            pulser.QPUBackend.validate_job_params(job_params, target_device.max_runs)

        # Submit one QRMI Job per job params
        new_job_ids: list[str] = []
        for params in job_params:
            seq_to_submit = sequence
            if sequence.is_parametrized() or sequence.is_register_mappable():
                variables = params.get("variables", {})
                seq_to_submit = sequence.build(**variables)
            assert not (
                seq_to_submit.is_parametrized() or seq_to_submit.is_register_mappable()
            )
            payload = Payload.PasqalCloud(
                sequence=seq_to_submit.to_abstract_repr(), job_runs=params["runs"]
            )
            task_id = self._qrmi.task_start(payload)
            logger.info("task start: %s", task_id)
            self._task_sequences[task_id] = seq_to_submit
            new_job_ids.append(task_id)

        batch_id = self._batch_id_from_job_ids(new_job_ids)

        if wait:
            for job_id in new_job_ids:
                self._wait_job_execution(job_id)

        return RemoteResults(batch_id=batch_id, connection=self, job_ids=new_job_ids)

    @staticmethod
    def _batch_id_from_job_ids(job_ids: list[str]) -> str:
        """Generate the Batch ID from a list of Job IDs."""
        return "|".join(job_ids)

    def _wait_job_execution(self, job_id: str):
        """Waits until job execution is complete"""
        while True:
            status = self._qrmi.task_status(job_id)
            if status in (
                TaskStatus.Completed,
                TaskStatus.Failed,
                TaskStatus.Cancelled,
            ):
                return
            time.sleep(JOB_EXECUTION_POLLING_INTERVAL_S)

    def _get_job_ids(self, batch_id: str) -> list[str]:
        """Retrieve the list of Job IDs from the Batch ID"""
        return batch_id.split("|")

    def _close_batch(self, batch_id: str) -> None:
        for job_id in self._get_job_ids(batch_id):
            self._qrmi.task_stop(job_id)

    @staticmethod
    def _to_job_status(status: TaskStatus) -> JobStatus:
        return _QRMI_TASK_STATUS_MAP.get(status, JobStatus.ERROR)

    def _task_result_to_results(self, task_id: str) -> Results:
        raw_result = self._qrmi.task_result(task_id).value
        parsed_result = _normalize_json_payload(raw_result)
        counter_payload = parsed_result.get("counter")
        if not isinstance(counter_payload, dict):
            raise RemoteResultsError(
                f"Unsupported counter payload for task {task_id!r}: {counter_payload!r}."
            )

        bitstring_counts = {
            str(bitstring): int(count)
            for bitstring, count in counter_payload.items()
            if isinstance(count, (int, float))
        }
        if not bitstring_counts:
            raise RemoteResultsError(
                f"No valid counts found in task {task_id!r} payload: {parsed_result!r}."
            )

        sequence = self._task_sequences.get(task_id)
        if sequence is None:
            raise RemoteResultsError(f"Missing sequence context for task {task_id!r}.")
        register = sequence.get_register(include_mappable=True)
        basis = sequence.get_measurement_basis()
        if basis is None:
            raise RemoteResultsError(f"Missing measurement basis for task {task_id!r}.")
        return SampledResult(  # pylint: disable=no-value-for-parameter
            atom_order=tuple(register.qubit_ids),
            meas_basis=basis,
            bitstring_counts=bitstring_counts,
        )
