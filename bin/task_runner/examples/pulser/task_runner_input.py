import json

from pulser import Pulse, Register, Sequence
from pulser.devices import DigitalAnalogDevice

reg = Register(
    {
        "q0": (-2.5, -2.5),
        "q1": (2.5, -2.5),
        "q2": (-2.5, 2.5),
        "q3": (2.5, 2.5),
    }
)

seq = Sequence(reg, DigitalAnalogDevice)
seq.declare_channel("rydberg", "rydberg_global")

pulse1 = Pulse.ConstantPulse(100, 2, 2, 0)

seq.add(pulse1, "rydberg")
seq.measure("ground-rydberg")

sequence = seq.to_abstract_repr()

program = {"sequence": sequence, "job_runs": 1000}

with open("./sequence.json", "w") as f:
    json.dump(program, f)
