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
import logging
import time
import typing
import uuid

import pulser
from pulser.backend.remote import (
    BatchStatus,
    JobParams,
    JobStatus,
    RemoteConnection,
    RemoteResults,
    RemoteResultsError,
)
from pulser.backend.results import Results
from pulser.result import SampledResult

from qrmi import Payload, QuantumResource, TaskStatus  # type: ignore

logger = logging.getLogger(__name__)


def _normalize_json_payload(payload: typing.Any) -> dict[str, typing.Any]:
    """Return payload as a dictionary from JSON text or dict."""
    if isinstance(payload, str):
        normalized = json.loads(payload)
    elif isinstance(payload, dict):
        normalized = payload
    else:
        raise TypeError("Unsupported payload type. Expected JSON string or dict.")

    if not isinstance(normalized, dict):
        raise TypeError("Invalid payload. Expected a JSON object.")

    return normalized


def _is_missing_device_specs_error(err: RuntimeError) -> bool:
    """Return True when QRMI reports device specs are unavailable."""
    message = str(err)
    start = message.find("{")
    if start < 0:
        return False
    try:
        payload = json.loads(message[start:])
    except json.JSONDecodeError:
        return False
    return payload.get("code") == "CD1202"


def _normalize_target_payload(target: typing.Any) -> dict[str, typing.Any]:
    """Return target payload as a dictionary.

    The target may be a QRMI wrapper object with `.value`, JSON text, or a dict.
    """
    target_value = target.value if hasattr(target, "value") else target
    return _normalize_json_payload(target_value)


class PulserQRMIConnection(RemoteConnection):
    """A connection to Pasqal QRMI resources, to submit Sequences to QPUs."""

    def __init__(self, qrmi: QuantumResource) -> None:
        self._qrmi = qrmi
        self._batch_job_ids: dict[str, list[str]] = {}
        self._task_sequences: dict[str, pulser.Sequence] = {}

    def supports_open_batch(self) -> bool:
        """Flag to confirm this class doesn't support open batch creation."""
        return False

    def fetch_available_devices(self) -> dict[str, pulser.devices.Device]:
        target = self._qrmi.target()
        target_payload = _normalize_target_payload(target)
        dev = pulser.abstract_repr.deserialize_device(json.dumps(target_payload))
        return {dev.name: dev}

    def update_sequence_device(self, sequence: pulser.Sequence) -> pulser.Sequence:
        """Match sequence device to remote specs when available."""
        try:
            return super().update_sequence_device(sequence)
        except RuntimeError as err:
            if not _is_missing_device_specs_error(err):
                raise
            logging.warning(
                "The selected connection doesn't give access to the latest "
                "device specs. Execution might fail if the sequence is "
                "incompatible with the device."
            )
            return sequence

    def _fetch_result(
        self, batch_id: str, job_ids: list[str] | None
    ) -> pulser.Sequence[Results]:
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
        statuses = [
            self._to_job_status(self._qrmi.task_status(job_id))
            for job_id in self._get_job_ids(batch_id)
        ]
        if not statuses:
            return BatchStatus.DONE
        if any(status == JobStatus.ERROR for status in statuses):
            return BatchStatus.ERROR
        if any(status == JobStatus.RUNNING for status in statuses):
            return BatchStatus.RUNNING
        if any(status == JobStatus.PENDING for status in statuses):
            return BatchStatus.PENDING
        if any(status == JobStatus.CANCELED for status in statuses):
            return BatchStatus.CANCELED
        return BatchStatus.DONE

    def _query_job_progress(
        self, batch_id: str
    ) -> typing.Mapping[str, tuple[JobStatus, Results | None]]:
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
                ) as err:
                    logger.warning(
                        "Failed to parse QRMI result for job %s: %s", job_id, err
                    )
            progress[job_id] = (status, result)
        return progress

    def submit(
        self,
        sequence: pulser.Sequence,
        wait: bool = True,
        open: bool = False,  # pylint: disable=redefined-builtin
        batch_id: str | None = None,
        **kwargs: typing.Any,
    ) -> RemoteResults | list[typing.Any]:
        """Submits the sequence for execution on a remote Pasqal backend.

        For compatibility with older QRMI Pulser examples:
        - `wait=False` returns `RemoteResults`
        - `wait=True` returns the legacy list of raw QRMI payloads.
        """
        if open:
            raise NotImplementedError("Open batches are not implemented in QRMI.")
        sequence = self._add_measurement_to_sequence(sequence)
        # Check that Job Params are correctly defined
        job_params: list[JobParams] = pulser.json.utils.make_json_compatible(
            kwargs.get("job_params", [])
        )
        mimic_qpu: bool = kwargs.get("mimic_qpu", False)
        if mimic_qpu:
            # Replace the sequence's device by the QPU's
            sequence = self.update_sequence_device(sequence)
            # Check that the job params match with the max number of runs
            pulser.QPUBackend.validate_job_params(job_params, sequence.device.max_runs)

        # In PasqalCloud, if batch_id is not empty, we can submit new jobs to a
        # batch we just created. This is not implemented in QRMI.
        if batch_id:
            raise NotImplementedError(
                "It is not possible to add jobs to a previously created batch "
                "with QRMI."
            )

        # Create a new batch by submitting to the targeted qpu.
        # If QRMI exposes device specs for the selected resource, enforce a
        # sequence-device match and validate runs against the target max_runs.
        try:
            available_devices = self.fetch_available_devices()
        except RuntimeError as err:
            if not _is_missing_device_specs_error(err):
                raise
            logger.debug("Skipping device/job-params validation: %s", err)
            available_devices = {}

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

        new_batch_id = new_job_ids[0] if len(new_job_ids) == 1 else str(uuid.uuid4())
        self._batch_job_ids[new_batch_id] = new_job_ids

        if not wait:
            return self.get_results(batch_id=new_batch_id, job_ids=new_job_ids)

        raw_results: list[typing.Any] = []
        for job_id in new_job_ids:
            while True:
                status = self._qrmi.task_status(job_id)
                if status == TaskStatus.Completed:
                    raw_results.append(self._qrmi.task_result(job_id).value)
                    break
                if status in (TaskStatus.Failed, TaskStatus.Cancelled):
                    break
                time.sleep(1)

        return raw_results

    def get_results(
        self,
        batch_id: str,
        job_ids: list[str] | None = None,
    ) -> RemoteResults:
        """Gets the results handle for a specific batch."""
        return RemoteResults(batch_id=batch_id, connection=self, job_ids=job_ids)

    def _get_job_ids(self, batch_id: str) -> list[str]:
        if batch_id in self._batch_job_ids:
            return list(self._batch_job_ids[batch_id])
        if batch_id in self._task_sequences:
            return [batch_id]
        raise RuntimeError(f"Batch {batch_id!r} is unknown to this QRMI connection.")

    def _close_batch(self, batch_id: str) -> None:
        for job_id in self._get_job_ids(batch_id):
            self._qrmi.task_stop(job_id)

    @staticmethod
    def _to_job_status(status: TaskStatus) -> JobStatus:
        status_map = {
            TaskStatus.Queued: JobStatus.PENDING,
            TaskStatus.Running: JobStatus.RUNNING,
            TaskStatus.Completed: JobStatus.DONE,
            TaskStatus.Failed: JobStatus.ERROR,
            TaskStatus.Cancelled: JobStatus.CANCELED,
        }
        return status_map.get(status, JobStatus.ERROR)

    def _task_result_to_results(self, task_id: str) -> Results:
        raw_result = self._qrmi.task_result(task_id).value
        parsed_result = _normalize_json_payload(raw_result)
        counter_payload = parsed_result.get("counter", parsed_result)
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
