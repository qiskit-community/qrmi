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

from __future__ import annotations

import json
import logging
import time
import typing

import pulser
from pulser.backend.remote import (
    BatchStatus,
    JobParams,
    JobStatus,
    RemoteBackend,
    RemoteConnection,
)
from pulser.backend.results import Results
from pulser.devices import Device

from qrmi import Payload, QuantumResource, TaskStatus  # type: ignore

logger = logging.getLogger(__name__)


class PulserQRMIConnection(RemoteConnection):
    """A connection to Pasqal Cloud, to submit Sequences to QPUs."""

    def __init__(self, qrmi: QuantumResource) -> None:
        self._qrmi = qrmi

    def supports_open_batch(self) -> bool:
        """Flag to confirm this class doesn't support open batch creation."""
        return False

    def fetch_available_devices(self) -> dict[str, pulser.devices.Device]:
        target = self._qrmi.target()
        target = json.loads(target.value)
        dev = Device(**target)
        return {dev.name: dev}

    def _fetch_result(
        self, batch_id: str, job_ids: list[str] | None
    ) -> pulser.Sequence[Results]:
        raise NotImplementedError("Not applicable to current design")

    def _get_batch_status(self, batch_id: str) -> BatchStatus:
        raise NotImplementedError("Not applicable to current design")

    def _query_job_progress(
        self, batch_id: str
    ) -> typing.Mapping[str, tuple[JobStatus, Results | None]]:
        raise NotImplementedError("Not applicable to current design")

    def submit(
        self,
        sequence: pulser.Sequence,
        wait: bool = True,
        open: bool = False,
        batch_id: str | None = None,
        **kwargs: typing.Any,
    ) -> list[dict[str, dict[str, str]]]:  # type: ignore
        """Submits the sequence for execution on a remote Pasqal backend.
        Currently forces waiting for execution and return list of result dicts.
        """
        if open:
            raise NotImplementedError("Open batches are not implemented in QRMI.")
        if not wait:
            raise NotImplementedError("Waiting is currently implied by QRMI")
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

        # TODO, reinstate check
        # Create a new batch by submitting to the targeted qpu
        # Find the targeted QPU
        # for qpu_id, device in self.fetch_available_devices().items():
        #    if sequence.device.name == device.name:
        #        break
        # else:
        #    raise ValueError(
        #        f"The Sequence's device {sequence.device.name} doesn't match the "
        #        "name of a device of any available QPU. Select your device among"
        #        "fetch_available_devices() and change your Sequence's device using"
        #        "its switch_device method."
        #    )

        # Check JobParams
        # pulser.QPUBackend.validate_job_params(job_params, device.max_runs)

        # Submit one QRMI Job per job params
        results = []
        for params in job_params:
            seq_to_submit = sequence
            if sequence.is_parametrized() or sequence.is_register_mappable():
                vars = params.get("variables", {})
                seq_to_submit = sequence.build(**vars)
            assert not (
                seq_to_submit.is_parametrized() or seq_to_submit.is_register_mappable()
            )
            payload = Payload.PasqalCloud(
                sequence=seq_to_submit.to_abstract_repr(), job_runs=params["runs"]
            )
            new_task_id = self._qrmi.task_start(payload)
            print(f"task start: {new_task_id}", flush=True)
            while wait:  # Currently always True
                status = self._qrmi.task_status(new_task_id)
                if status == TaskStatus.Completed:
                    time.sleep(0.5)
                    # Get the results
                    results.append(self._qrmi.task_result(new_task_id).value)
                    break
                elif status == TaskStatus.Failed:
                    break
                else:
                    print("Task status %s, waiting 1s" % status, flush=True)
                    time.sleep(1)
        return results


class PulserQRMIBackend(RemoteBackend):
    def __init__(
        self,
        sequence: pulser.Sequence,
        connection: PulserQRMIConnection,
        mimic_qpu: bool = False,
    ) -> None:
        self._sequence = sequence
        self._connection = connection
        self._mimic_qpu = mimic_qpu

    def run(self, job_params: list[JobParams] | None = None, wait: bool = False) -> list[dict[str, dict[str, str]]]:  # type: ignore
        return self._connection.submit(self._sequence, wait=wait, job_params=job_params, mimic_qpu=self._mimic_qpu)  # type: ignore
