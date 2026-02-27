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
$ pip install cudaq
```

## Set environment variables

QRMI supports Pasqal Cloud configuration via environment variables.
For Pasqal Cloud auth, QRMI also supports reading `~/.pasqal/config` (token or username/password).

The required environment variables are listed below. They are populated automatically by the spank plugin.

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

When CUDA-Q is configured with target `pasqal` and  `machine` in `cudaq.set_target(..., machine=...)` should be
match `qrmi`.

```python
import cudaq
cudaq.set_target("pasqal", machine="qrmi")
```

See the CUDA-Q docs too see how to send a C++ job. To use QRMI, simply set the target and machine as above.

### For development builds

For development builds, follow CUDA-Q's official build docs and scripts.
For full toolchain details, use CUDA-Q docs/scripts as source of truth.

```bash
# 1) Rebuild QRMI
cd /shared/qrmi
cargo build --release --lib

# 2) Build CUDA-Q + prerequisites via CUDA-Q scripts
cd /shared/cuda-quantum
bash scripts/build_cudaq.sh -p -i -j nproc -- \
  -DQRMI_ROOT=/shared/qrmi \ # Required!
  -DCUDAQ_BUILD_TESTS=OFF \
  -DCUDAQ_ENABLE_BRAKET_BACKEND=OFF \
  -DCUDAQ_ENABLE_QCI_BACKEND=OFF \
  -DCUDAQ_ENABLE_QUANTUM_MACHINES_BACKEND=OFF

# We can keep on disabling more backends too to speed up the compilation.

# 3) Reinstall CUDA-Q Python package (non-editable!)
source /shared/pyenv/bin/activate
pip uninstall -y cuda-quantum-cu13 || true
pip install --no-build-isolation /shared/cuda-quantum
```

Do not use editable install for CUDA-Q in this workspace (`pip install -e .`) as it requires further manually specifying paths to get a working environment.

## How to run

All information is baked into the Python script.

```shell-session

For example,
```shell-session
$ python pasqal.py
```
