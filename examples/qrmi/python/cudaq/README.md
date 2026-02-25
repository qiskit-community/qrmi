# Pasqal Cloud QRMI - CUDA-Q Examples

## Prerequisites

* Rust 1.85.1 or above
* Python 3.11 or 3.12
* [QRMI python package installation](../../../../README.md)
* CUDA-Q installed with the pasqal backend built

## Install dependencies

```shell-session
$ source ~/py311_qrmi_venv/bin/activate
$ pip install -r ../requirements.txt
```

## Set environment variables

QRMI supports Pasqal Cloud configuration via environment variables.
For Pasqal Cloud auth, QRMI also supports reading `~/.pasqal/config` (token or username/password).

The required environment variables are listed below. This example assumes that a `.env` file is available under the current directory.

| Environment variables | Descriptions |
| ---- | ---- |
| <backend_name>_QRMI_PASQAL_CLOUD_PROJECT_ID | Pasqal Cloud Project ID to access the QPU |
| <backend_name>_QRMI_PASQAL_CLOUD_AUTH_TOKEN | Pasqal Cloud Auth Token |
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

## Using this backend from CUDA-Q (`pasqal`)

When CUDA-Q is configured with target `pasqal`, QRMI is used as the
Pasqal cloud bridge. `machine` in `cudaq.set_target(..., machine=...)` should
match `<backend_name>` above (for example `EMU_FREE`).

```python
import cudaq
cudaq.set_target("pasqal", machine="EMU_FREE")
```

### For development builds
When installing pre-built binaries in supported ways, the below steps should not be needed.

However, building CUDA-Q and QRMI concurrently means that one has to set up the environment more carefully.
For development, assuming the instructions in demo(insertPath)/INSTALL.md are followed, we then 
clone the CUDA-Q repository into slurm-docker-cluster/shared.

We then have to apply the patch `cuda-q-dev.patch` in this directory.
```bash
patch -p1 ./cuda-q-dev.patch
```

At runtime, CUDA-Q's Pasqal QRMI plugin library and `libqrmi.so` must be
discoverable, which we can achieve by setting the following environment variables:

```bash
CUDAQ_SITE_PACKAGES="$(python - <<'PY'
import pathlib
import cudaq
print(pathlib.Path(cudaq.__file__).resolve().parent.parent)
PY
)"
export LD_LIBRARY_PATH="${CUDAQ_SITE_PACKAGES}/lib:/path/to/qrmi/target/release:${LD_LIBRARY_PATH:-}"
export CUDAQ_DYNLIBS="${CUDAQ_SITE_PACKAGES}/lib/libcudaq-pasqal-qpu.so"
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
