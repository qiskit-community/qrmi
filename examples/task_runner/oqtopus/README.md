# Tools to generate OQTOPUS job spec JSON input from Qiskit QuantumCircuit

The tools demonstrate the generation of OQTOPUS job spec JSON input from a quantum circuit example.

## Prerequisites
* Python 3.11 or above


## Install dependencies

```shell-session
pip install -f requirements.txt
```

## Tools

### gen_sampling_inputs.py

Generates OQTOPUS job spec JSON input for the bell-state circuit.


Usage:
```shell-session
usage: gen_sampling_inputs.py [-h] device_id

A tool to generate OQTOPUS job spec JSON input from Bell-state QuantumCircuit

positional arguments:
  device_id   device ID

options:
  -h, --help  show this help message and exit
```

Example:
```bash
python gen_sampling_inputs.py qulacs
```
