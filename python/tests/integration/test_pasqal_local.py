# This code is part of Qiskit.
#
# (C) Copyright 2026 Pasqal. All Rights Reserved.
#
# This code is licensed under the Apache License, Version 2.0. You may
# obtain a copy of this license in the LICENSE.txt file in the root directory
# of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
#
# Any modifications or derivative works of this code must retain this
# copyright notice, and modified files need to carry a notice indicating
# that they have been altered from the originals.

"""End-to-end tests for Pasqal Local against Warden and its mock QPU.

Requires `munged`, a running Warden and a mock QPU started with ``MOCK_QPU_API_EMUL`` set,
so submitted sequences are emulated with QuTiP. Also requires these environment variables:
``PASQAL_LOCAL_QRMI_WARDEN_URL``, ``QRMI_JOB_UID`` and ``QRMI_JOB_ID``.
"""

import os
import time

import numpy as np
import pulser
import pytest
from pulser import Pulse, Register, Sequence
from pulser.backend.remote import JobParams

from qrmi import Payload, QuantumResource, ResourceStatusCode, ResourceType, TaskStatus
from qrmi.pulser.connection import PulserQRMIConnection

RESOURCE = "PASQAL_LOCAL"


@pytest.fixture(name="qrmi", scope="module")
def fixture_qrmi():
    """Acquire a Warden session for the module and release it afterwards."""
    qrmi = QuantumResource(RESOURCE, ResourceType.PasqalLocal)
    token = qrmi.acquire()
    os.environ[f"{RESOURCE}_QRMI_JOB_ACQUISITION_TOKEN"] = token
    yield qrmi
    qrmi.release(token)


@pytest.fixture(name="sequence", scope="module")
def fixture_sequence(qrmi):
    """Build a pi-pulse on two non-interacting atoms, on the device reported by Warden."""
    device = next(iter(PulserQRMIConnection(qrmi).fetch_available_devices().values()))
    reg = Register.from_coordinates([(0, 0), (20, 0)], prefix="q")
    seq = Sequence(reg.with_automatic_layout(device), device)
    seq.declare_channel("rydberg", "rydberg_global")
    seq.add(Pulse.ConstantPulse(500, 2 * np.pi, 0, 0), "rydberg")
    seq.measure("ground-rydberg")
    return seq


def test_status_online(qrmi):
    """Warden reports the mock QPU as online."""
    assert qrmi.status().status == ResourceStatusCode.Online


def test_task_stop(qrmi, sequence):
    """A submitted task can be cancelled."""
    task_id = qrmi.task_start(
        Payload.PasqalCloud(sequence=sequence.to_abstract_repr(), job_runs=1000)
    )
    qrmi.task_stop(task_id)
    # Warden cancels a job the scheduler already picked up in the background.
    deadline = time.monotonic() + 10
    while (status := qrmi.task_status(task_id)) != TaskStatus.Cancelled:
        assert time.monotonic() < deadline, f"task still {status} after task_stop"
        time.sleep(0.5)


def test_pulser_backend(qrmi, sequence):
    """Pulser's QPUBackend runs a sequence to completion through QRMI."""
    conn = PulserQRMIConnection(qrmi)
    remote_results = pulser.QPUBackend(sequence, conn).run(
        [JobParams(runs=100, variables={})], wait=True
    )
    (result,) = remote_results.results
    counts = result.final_bitstrings
    assert sum(counts.values()) == 100
    # A pi-pulse excites both non-interacting atoms.
    assert counts.get("11", 0) > 50
    assert conn.get_batch_logs(batch_id=remote_results.batch_id)
