from qsharp import TargetProfile
from qsharp.interop.qiskit import QSharpBackend
from qiskit import QuantumCircuit

# Define Circuit
circuit = QuantumCircuit(1, 1)
circuit.delay(100, unit="us") # Seems to not be taken into account by the QIR conversion
circuit.measure(0, 0)

backend = QSharpBackend()
print(backend.qir(circuit, target_profile=TargetProfile.Adaptive_RI))