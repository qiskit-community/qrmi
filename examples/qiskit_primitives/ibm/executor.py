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

"""Executor example with IBM Quantum System QRMI"""

# pylint: disable=invalid-name
import random
from dotenv import load_dotenv
from qiskit.circuit import Parameter, QuantumCircuit
from qiskit.transpiler import generate_preset_pass_manager
from qrmi.primitives import QRMIService
from qrmi.primitives.ibm import Executor, get_backend
from qiskit_ibm_runtime.quantum_program import QuantumProgram
import numpy as np
from samplomatic import build
from samplomatic.transpiler import generate_boxing_pass_manager

# Create QRMI
load_dotenv()
service = QRMIService()

resources = service.resources()
if len(resources) == 0:
    raise ValueError("No quantum resource is available.")

for res in resources:
    print(f"Available resource: id={res.resource_id()} type={str(res.resource_type())}")

# Randomly select QR
qrmi = resources[random.randrange(len(resources))]
print(qrmi.metadata())

# Generate Qiskit BackendV2 instance from QuantumResource
backend = get_backend(qrmi)

# Generate the circuit
circuit = QuantumCircuit(3)
circuit.h(0)
circuit.h(1)
circuit.cz(0, 1)
circuit.h(1)
circuit.h(2)
circuit.cz(1, 2)
circuit.h(2)
circuit.rz(Parameter("theta"), 0)
circuit.rz(Parameter("phi"), 1)
circuit.rz(Parameter("lam"), 2)
circuit.measure_all()

# Transpile the circuit to ISA
preset_pass_manager = generate_preset_pass_manager(
    backend=backend, optimization_level=3
)
isa_circuit = preset_pass_manager.run(circuit)

boxing_pm = generate_boxing_pass_manager(
    # Add gate twirling
    enable_gates=True,
    # Add measurement twirling
    enable_measures=True,
)

boxed_circuit = boxing_pm.run(isa_circuit)

# Build the template circuit and the samplex
template_circuit, samplex = build(boxed_circuit)

# Generate a quantum program
program = QuantumProgram(shots=1024)

# Append the circuit and the parameter values to the program
program.append_circuit_item(
    isa_circuit,
    circuit_arguments=np.random.rand(10, 3),  # 10 sets of parameter values
)

# Append the template circuit and samplex as a samplex item
program.append_samplex_item(
    template_circuit,
    samplex=samplex,
    samplex_arguments={
        "parameter_values": np.random.rand(10, 3),  # 10 sets of parameter values
    },
    shape=(2, 14, 10),
)

# initialize an Executor with default options
executor = Executor(mode=backend)

# Submit the job
job = executor.run(program)

# Retrieve the result
result = job.result()

# Access the results of the classical register of task #0, the CircuitItem
result_0 = result[0]["meas"]
print(result_0)

# Access the results of the classical register of task #1, the SamplexItem
result_1 = result[1]["meas"]
print(result_1)

# Access the bit-flip corrections
flips_1 = result[1]["measurement_flips.meas"]
print(f"Bit-flip corrections shape: {flips_1.shape}")

# Undo the bit flips via classical XOR
unflipped_result_1 = result_1 ^ flips_1

job.result().timing.draw()
