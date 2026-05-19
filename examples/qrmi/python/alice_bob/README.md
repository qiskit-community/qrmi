# Alice and Bob Felis QRMI - Examples in Python

## Prerequisites

* Rust 1.85.1 or above
* Python 3.11 or 3.12
* [QRMI python package installation](../../../../README.md)

## Install dependencies

```shell-session
pip install -r ../requirements.txt
```

## Set environment variables

Because QRMI is an environment variable driven software library, all configuration parameters must be specified in environment variables. The required environment variables are listed below. You can either create a `.env` file in the current directory or set them in the shell directly.

| Environment variables | Descriptions |
| ---- | ---- |
| QRMI_AB_FELIS_BASE_ENDPOINT| Felis URL |
| QRMI_AB_FELIS_API_KEY | Felis API Key |

You may also use separate URLs and API keys for different target resources:

| Environment variables | Descriptions |
| ---- | ---- |
| {resource_name}_QRMI_AB_FELIS_BASE_ENDPOINT| Felis URL |
| {resource_name}_QRMI_AB_FELIS_API_KEY | Felis API Key |

## Generate QIR input file

Felis accepts circuits expressed in human-readable QIR, a language which expresses intermediate representations of quantum circuits. It supports standard QIR gates together with a few custom Alice & Bob gates.

A trivial QIR circuit can be generated with:

```
cd examples/qrmi/python/alice_bob/
pip install qsharp
python generate_input.py > generated_circuit.ll
```

If you want to run circuits which use native Alice & Bob gates (`initialize` and `measure_x`), you will also need to install Alice and Bob's Qiskit provider:

```shell-session
pip install qiskit-alice-bob-provider
```

⚠️ Warning: Installing `qiskit-alice-bob-provider` will downgrade `qiskit` to version 1.x, which could break other vendor implementations. You may want to use a separate python environment for this step.

Then, you can optionally modify the Qiskit circuit in `generate_input_native_gates.py` before generating it in QIR form as follows:

```
export QRMI_AB_FELIS_BASE_ENDPOINT='<felis endpoint>'
export QRMI_AB_FELIS_API_KEY='<your felis api key>'
python generate_input_native_gates.py EMU:1Q:LESCANNE_2020 > generated_circuit.ll
```

The environment variables are necessary for transpilation, which requires access to the Felis API.

## How to run

```shell-session
$ python example.py -h
usage: example.py [-h] target qir_file

An example of a Quantum Resource from Alice and Bob's Felis API

positional arguments:
  target      backend name
  qir_file    qir input file

options:
  -h, --help  show this help message and exit
```

For example,

```shell-session
export ab_emu_1q_lescanne_2020_QRMI_AB_FELIS_BASE_ENDPOINT='https://api.alice-bob.com/'
export ab_emu_1q_lescanne_2020_QRMI_AB_FELIS_API_KEY='<your felis api key>'

python example.py 'ab_emu_1q_lescanne_2020' generated_circuit.ll
```
