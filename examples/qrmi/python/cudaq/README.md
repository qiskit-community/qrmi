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
match `qrmi`. This way it picks up the machine target from QRMI, as populated by e.g. the Spank plugin's
`--qpu` argument, or manually set by `SLURM_JOB_QPU_RESOURCES`.

```python
import cudaq
cudaq.set_target("pasqal", machine="qrmi")
```

See the CUDA-Q docs too see how to send a C++ job. To use QRMI, simply set the target and machine as above.

## How to run

All information is baked into the Python script.

```shell-session

For example,
```shell-session
$ python pasqal.py
```

## Build from source

> For up-to-date  information on how to build the latest version, we suggest you follow CUDA-Q's [official build docs and scripts](https://nvidia.github.io/cuda-quantum/latest/using/install/data_center_install.html).

We assume Slurm containers as setup by the [spank-plugins development INSTALL.md](https://github.com/qiskit-community/spank-plugins/blob/main/demo/qrmi/slurm-docker-cluster/INSTALL.md) and the cudaq repo cloned in `/shared`.

```bash
# 1) Rebuild QRMI
cd /shared/qrmi
cargo build --release --lib
# For pasqal-local support
cargo build --release --lib --features munge

# 2) Build CUDA-Q with local QRMI
cd /shared/cuda-quantum
QRMI_INSTALL_PREFIX=/shared/qrmi bash scripts/build_cudaq.sh -p -i -j nproc

# 3) Install CUDA-Q Python package (non-editable!)
source /shared/pyenv/bin/activate
pip uninstall -y cuda-quantum-cu13 || true
pip install --no-build-isolation /shared/cuda-quantum
```

Do not use editable install for CUDA-Q in this workspace (`pip install -e .`) as it requires further manually specifying paths to get a working environment.

> The CUDA-Q build config used during development were these:
```bash
dnf install epel-release
dnf makecache
dnf install ccache
source /shared/pyenv/bin/activate && cd /shared/cuda-quantum
PATH=/opt/llvm/bin:$PATH Python3_EXECUTABLE=/shared/pyenv/bin/python ./scripts/install_prerequisites.sh -e "aws;qrmi"
PATH=/opt/llvm/bin:$PATH Python3_EXECUTABLE=/shared/pyenv/bin/python QRMI_INSTALL_PREFIX=/shared/qrmi CUDAQ_BUILD_TESTS=FALSE CUDAQ_WERROR=OFF ./scripts/build_cudaq.sh -j nproc -- -DCUDAQ_ENABLE_PASQAL_QRMI_CONNECTOR=ON -DCUDAQ_ENABLE_BRAKET_BACKEND=OFF -DCUDAQ_ENABLE_QCI_BACKEND=OFF -DCUDAQ_ENABLE_QUANTUM_MACHINES_BACKEND=OFF
```

### Troubleshooting

To be sure that Cuda-Q detected and is using the QRMI lib that you just built, checkout the `QRMI_LIBRARY` var in `cuda-quantum/build/CMakeCache.txt`. By default, that QRMI lib build is located in `qrmi/target/release/libqrmi.so`, so you can copy it to where `QRMI_LIBRARY` is pointing if there is a mismatch.
