# Tools to generate OQTOPUS program input from Qiskit QuantumCircuit

The tools demonstrate the generation of OQTOPUS program input from a quantum circuit example.

## Prerequisites
* Python 3.11 or above


## Install dependencies

```shell-session
pip install -f requirements.txt
```

## Tools

### gen_sampling_inputs.py

Generates OQTOPUS program input for the bell-state circuit.


Usage:
```shell-session
usage: gen_sampling_inputs.py [-h] output

A tool to generate OQTOPUS program input from Bell-state QuantumCircuit

positional arguments:
  output      output file

options:
  -h, --help  show this help message and exit
```

Example:
```bash
python gen_sampling_inputs.py bell_state.txt
```
