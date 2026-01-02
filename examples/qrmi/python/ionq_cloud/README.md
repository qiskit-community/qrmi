# IonQ Cloud QRMI - Examples in Python

## Prerequisites

* Rust 1.85.1 or above
* Python 3.11 or 3.12
* [QRMI python package installation](../../../../README.md)

## Install dependencies

```shell-session
$ source ~/py311_qrmi_venv/bin/activate
$ pip install -r ../requirements.txt
```

## Set environment variables

Because QRMI is an environment variable driven software library, all configuration parameters must be specified in environment variables. The required environment variables are listed below. This example assumes that a `.env` file is available under the current directory.

| Environment variables      | Description              |
|----------------------------|--------------------------|
| QRMI_IONQ_CLOUD_API_KEY    | IonQ Cloud API key       |

## Create and input file with JSON data:

Checkout this resource for input data format: https://docs.ionq.com/api-reference/v0.4/introduction
For the purpose of testing save this data as a file named job.json:

```{
  "type" : "ionq.circuit.v1",
  "name": "Sample circuit",
  "metadata": {
    "fizz": "buzz",
    "foo": "bar"
  },
  "settings" :
  {
    "error_mitigation":
    {
      "debiasing": false
    }
  },
  "noise": {
      "model": "aria-1"
  },
  "input": {
    "qubits":  2,
    "gateset": "qis",
    "circuit": [
    {
      "gate": "h",
      "target": 0
    }
    ]
  }
}
```

## How to run

```shell-session
$ python example.py -h
usage: example.py [-h] backend shots input

An example of IonQ Cloud QRMI

positional arguments:
  backend     backend name
  shots       number of shots
  input       job input file

options:
  -h, --help  show this help message and exit
```

For example:

```shell-session
$ python example.py simulator 10000 job.json
```

