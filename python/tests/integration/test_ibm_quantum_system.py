# This code is part of Qiskit.
#
# (C) Copyright 2026 IBM, Pasqal. All Rights Reserved.
#
# This code is licensed under the Apache License, Version 2.0. You may
# obtain a copy of this license in the LICENSE.txt file in the root directory
# of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
#
# Any modifications or derivative works of this code must retain this
# copyright notice, and modified files need to carry a notice indicating
# that they have been altered from the originals.

"""End-to-end tests for IBM Quantum System against Simulated Quantum Resource.

Requires a running simulator (https://github.com/qrmi-community/simulated_quantum_resource)
and the ``fake_lagos_QRMI_IBM_QS_*`` and ``fake_lagos_QRMI_JOB_TIMEOUT_SECONDS``
environment variables.
"""

import pytest
from qiskit import QuantumCircuit
from qiskit.transpiler.preset_passmanagers import generate_preset_pass_manager

from qrmi import QuantumResource, ResourceStatusCode, ResourceType
from qrmi.primitives.ibm import SamplerV2, get_target

RESOURCE = "fake_lagos"


@pytest.fixture(name="qrmi", scope="module")
def fixture_qrmi():
    """Create the IBM Quantum System resource."""
    return QuantumResource(RESOURCE, ResourceType.IBMQuantumSystem)


def test_status_online(qrmi):
    """The simulator reports the backend as online."""
    assert qrmi.status().status == ResourceStatusCode.Online


def test_sampler(qrmi):
    """SamplerV2 runs a Bell circuit and returns results and logs through S3."""
    qc = QuantumCircuit(2, 2)
    qc.h(0)
    qc.cx(0, 1)
    qc.measure([0, 1], [0, 1])
    isa_circuit = generate_preset_pass_manager(
        optimization_level=1, target=get_target(qrmi)
    ).run(qc)

    job = SamplerV2(qrmi).run([isa_circuit], shots=1024)
    counts = job.result()[0].data.c.get_counts()
    assert sum(counts.values()) == 1024
    assert qrmi.task_logs(job.job_id())
