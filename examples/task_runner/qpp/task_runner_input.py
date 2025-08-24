import json

from pulser import DigitalAnalogDevice
from qiskit.circuit import QuantumCircuit
from qiskit_pasqal_provider.providers.gate import HamiltonianGate, InterpolatePoints
from qiskit_pasqal_provider.providers.pulse_utils import (
    gen_seq,
    get_register_from_circuit,
)

# We define the coordinates of the atoms, 6 in total.
coords = [[0, 0], [3, 5.2], [6, 0], [9, -5.2], [9, 5.2], [12, 0]]

# With a blockade radius of 8.7
blockade_radius = 8.7

# Calculate interaction strength between nearest-neighbours
interaction = 5420158.53 / blockade_radius**6

# Set up an adiabatic pulse,
# This pulse ramps from up 0 -> 4, stays constant, and ramps down again during the times
times = [0, 0.2, 0.8, 1]
ampl = InterpolatePoints(values=[0, 4, 4, 0], times=times)
det = InterpolatePoints(
    values=[-10, -10, interaction / 2, interaction / 2],
    times=times,
)
phase = 0.0

# analog gate
gate = HamiltonianGate(ampl, det, phase, coords, grid_transform="triangular")

# Qiskit circuit with analog gate
qc = QuantumCircuit(len(coords))
qc.append(gate, qc.qubits)


# get the register from the analog gate inside QuantumCircuit
_analog_register = get_register_from_circuit(qc)

seq = gen_seq(
    analog_register=_analog_register,
    device=DigitalAnalogDevice,
    circuit=qc,
)

sequence = seq.to_abstract_repr()

program = {"sequence": sequence, "job_runs": 1000}

with open("./sequence.json", "w") as f:
    json.dump(program, f)
