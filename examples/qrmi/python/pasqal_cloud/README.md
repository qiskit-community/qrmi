# Pasqal Cloud QRMI - Examples in Python

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

QRMI supports Pasqal Cloud configuration via environment variables. For Pasqal Cloud auth, QRMI also supports reading `~/.pasqal/config` (token or username/password).

The required environment variables are listed below. This example assumes that a `.env` file is available under the current directory.

| Environment variables | Descriptions |
| ---- | ---- |

| <backend_name>_QRMI_PASQAL_CLOUD_PROJECT_ID` |  Pasqal Cloud Project ID to access the QPU
| <backend_name>_QRMI_PASQAL_CLOUD_AUTH_TOKEN | Pasqal Cloud Auth Token
| <backend_name>_QRMI_PASQAL_CLOUD_AUTH_ENDPOINT | (Optional) Auth endpoint URL/path for token retrieval. Default: `authenticate.pasqal.cloud/oauth/token` |

### ~/.pasqal/config (optional)

Create `~/.pasqal/config`:
```
username=<your username>
password=<your password>
# or:
# token=<your token>

# optional override:
# project_id=<your project id>
# auth_endpoint=<auth endpoint URL/path>
```

Backend-scoped keys are supported and override global keys:
```
FRESNEL.token=<your token>
FRESNEL.project_id=<your project id>
```

## Create Pulser Sequence file as input

Given a Pulser sequence `sequence`, we can convert it to a JSON string and write it to a file like this:

```python
serialized_sequence = sequence.to_abstract_repr()

with open("pulser_seq.json", "w") as f:
    f.write(serialized_sequence)
```

## How to run

```shell-session
$ python example.py -h
usage: example.py [-h] input backend

An example of Pasqal Cloud Python QRMI

positional arguments:
  backend  'FRESNEL'
  input       sequence input file

options:
  -h, --help  show this help message and exit
```
For example,
```shell-session
$ python example.py FRESNEL input.json
```
