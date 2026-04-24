import argparse
from dotenv import load_dotenv

from helpers import FelisQIRTranspiler
from qiskit import QuantumCircuit

load_dotenv()

parser = argparse.ArgumentParser(
    description="An example of a Quantum Resource from Alice and Bob's Felis API"
)
parser.add_argument("target", help="Felis target e.g. 'EMU:1Q:LESCANNE_2020'")

args = parser.parse_args()

# Define Circuit
circuit = QuantumCircuit(1, 1)
circuit.initialize('+')
circuit.measure_x(0, 0)

transpiler = FelisQIRTranspiler(args.target)
qir_circuit = transpiler.transpile(circuit)
print(qir_circuit)