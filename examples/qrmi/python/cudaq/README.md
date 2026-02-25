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

Use CUDA-Q scripts directly for local rebuilds:

```bash
# 1) Rebuild QRMI
cd /shared/qrmi
cargo build --release --lib

# 2) Rebuild CUDA-Q with QRMI integration
cd /shared/cuda-quantum
export PATH=/opt/llvm/bin:$PATH
export LLVM_INSTALL_PREFIX=/root/.llvm-project/build
bash scripts/build_cudaq.sh -i -j nproc -- \
  -DQRMI_ROOT=/shared/qrmi \
  -DLLVM_DIR=/root/.llvm-project/build/lib/cmake/llvm \
  -DMLIR_DIR=/root/.llvm-project/build/lib/cmake/mlir \
  -DClang_DIR=/root/.llvm-project/build/lib/cmake/clang \
  -DZLIB_ROOT=/usr/local/zlib \
  -DZLIB_LIBRARY=/usr/lib64/libz.so \
  -DZLIB_INCLUDE_DIR=/usr/include \
  -DOPENSSL_ROOT_DIR=/usr \
  -DOPENSSL_CRYPTO_LIBRARY=/usr/lib64/libcrypto.so \
  -DOPENSSL_SSL_LIBRARY=/usr/lib64/libssl.so \
  -DOPENSSL_INCLUDE_DIR=/usr/include \
  -DCUDAQ_BUILD_TESTS=OFF \
  -DCUDAQ_ENABLE_BRAKET_BACKEND=OFF \
  -DCUDAQ_ENABLE_QCI_BACKEND=OFF \
  -DCUDAQ_ENABLE_QUANTUM_MACHINES_BACKEND=OFF

We can keep on disabling more backends too to speed up the compilation.

# 3) Reinstall CUDA-Q Python package (non-editable!)
source /shared/pyenv/bin/activate
pip uninstall -y cuda-quantum-cu13 || true
pip install --no-build-isolation /shared/cuda-quantum
```

Do not use editable install for CUDA-Q in this workspace (`pip install -e .`).
It requires manually specifying paths to get a working environment.

Runtime setup (single supported configuration):

```bash
CUDAQ_SITE_PACKAGES="$(python - <<'PY'
import pathlib, cudaq
print(pathlib.Path(cudaq.__file__).resolve().parent.parent)
PY
)"
export LD_LIBRARY_PATH="${CUDAQ_SITE_PACKAGES}/lib:/shared/qrmi/target/release:${LD_LIBRARY_PATH:-}"
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
