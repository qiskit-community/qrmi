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

QRMI supports Pasqal Cloud configuration via environment variables. For Pasqal Cloud auth, QRMI also supports reading `~/.pasqal/config` (token or username/password). `PASQAL_CONFIG_ROOT` may point elsewhere and takes priority over `<backend_name>_PASQAL_CONFIG_ROOT`; QRMI expands `~`, `$VAR`, and `${VAR}` before appending `.pasqal/config`.  # pragma: allowlist secret

The required environment variables are listed below. This example assumes that a `.env` file is available under the current directory.

| Environment variables | Descriptions |
| ---- | ---- |
| <backend_name>_QRMI_PASQAL_CLOUD_PROJECT_ID | Pasqal Cloud Project ID to access the QPU |
| <backend_name>_QRMI_PASQAL_CLOUD_AUTH_TOKEN | Pasqal Cloud Auth Token (optional when username/password are configured) |
| <backend_name>_QRMI_PASQAL_CLOUD_CLIENT_ID | Pasqal Cloud service account client ID (optional) |
| <backend_name>_QRMI_PASQAL_CLOUD_CLIENT_SECRET | Pasqal Cloud service account client secret (optional) |
| <backend_name>_QRMI_PASQAL_CLOUD_AUTH_ENDPOINT | (Optional) Auth endpoint URL/path for token retrieval. Default: `authenticate.pasqal.cloud/oauth/token` |
| PASQAL_USERNAME | Pasqal Cloud username (optional, user-provided) |
| PASQAL_PASSWORD | Pasqal Cloud password (optional, user-provided) |

### ~/.pasqal/config (optional)

Create `~/.pasqal/config`:
```
username=<your username>
password=<your password>
# or:
# token=<your token>
# or:
# client_id=<your client id>
# client_secret=<your client secret>  # pragma: allowlist secret

# optional override:
# project_id=<your project id>
# auth_endpoint=<auth endpoint URL/path>
```

## Alternative: create the resource from a config map

Since QRMI v0.25.0, a resource can also be built from an explicit config dict
instead of environment variables, via `QuantumResource.from_config()`.
Unlike the environment-variable-driven constructor, this does **not** fall
back to environment variables, and only reads `~/.pasqal/config` if
`PASQAL_CONFIG_ROOT` or `HOME` is set explicitly in the config dict:

```python
from qrmi import QuantumResource, ResourceType

qrmi = QuantumResource.from_config(
    "FRESNEL",
    ResourceType.PasqalCloud,
    {
        "QRMI_PASQAL_CLOUD_PROJECT_ID": "your_project_id",
        "QRMI_PASQAL_CLOUD_AUTH_TOKEN": "your_auth_token",
    },
)
```

See the [0.25.0 migration guide](../../../../docs/migration/0.25.0.md) for
the full set of required/optional keys per resource type.

## Using this backend from CUDA-Q (`pasqal`)

When CUDA-Q is configured with target `pasqal`, QRMI is used as the
Pasqal cloud bridge. `machine` in `cudaq.set_target(..., machine=...)` should
match `<backend_name>` above (for example `EMU_FREE`).

```python
import cudaq
cudaq.set_target("pasqal", machine="EMU_FREE")
```

For CUDA-Q build/runtime details in this workspace, see:
- [`../cudaq/README.md`](../cudaq/README.md)

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
