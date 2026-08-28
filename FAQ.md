# Frequently Asked Questions

## Table of Contents

1. [General Question](#general-questions)
2. [Job Execution Errors](#job-execution-errors)
    1. [I get an error `error: spank_qrmi_c, failed to acquire resource: ibm_brisbane`](#i-get-an-error-error-spank_qrmi_c-failed-to-acquire-resource-ibm_brisbane)
3. [Release and Deployment](#release-and-deployment)
    1. [How can I check which version of QRMI is linked into a binary?](#how-can-i-check-which-version-of-qrmi-is-linked-into-a-binary)

## Job Execution Errors

### I get an error `error: spank_qrmi_c, failed to acquire resource: ibm_brisbane`

**Cause:** This error occurs when accessing IBM Quantum backends using an Open Plan account on IBM Quantum Platform.

**What to check:**

1. Setup

```bash
python3.11 -m venv ~/{your_pyenv}
source ~/{your_pyenv}/bin/activate
pip install qiskit-ibm-runtime
```

2. Create `test.py`

Replace SERVICE_CRN and API_KEY values with your credentials, <your backend name> with your backend name.


```python
"""A testcase to check if Qiskit Session can be created with the given credentials"""
from qiskit_ibm_runtime import QiskitRuntimeService, Session

SERVICE_CRN="YOUR_SERVICE_CRN"
API_KEY="YOUR_APIKEY"

service = QiskitRuntimeService(
    channel="ibm_cloud",
    instance=SERVICE_CRN,
    token=API_KEY,
)

backend = service.backend("<your backend name>")
with Session(backend=backend, max_time=1) as session:
    print("Succeeded in obtaining a Qiskit Session")
```

3. Run this testcase

```bash
python test.py
```

This will fail due to the error like:
```bash
You are not authorized to run a session when using the open plan.
```


**How to resolve:**

- Use a Premium Plan account, or
- Use [Batch](https://quantum.cloud.ibm.com/docs/en/guides/execution-modes#batch-mode) execution mode
  - Add `QRMI_IBM_QRS_SESSION_MODE` environment variable with "batch" in your qrmi_config.json

```json
    {
      "name": "ibm_brisbane",
      "type": "qiskit-runtime-service",
      "environment": {
          ... 
          "QRMI_IBM_QRS_SESSION_MODE": "batch"
      }
   }
```

## Release and deployment

### How can I check which version of QRMI is linked into a binary?

Every build of QRMI embeds its own crate version and git commit hash
directly into the compiled artifact (shared library, static library, or
any binary that links it), so you can check it without running the code.

#### Using `strings`

```shell-session
$ strings /path/to/spank_qrmi.so | grep QRMI_BUILD_VERSION
QRMI_BUILD_VERSION:0.24.0;QRMI_GIT_HASH:0dac1793b013
```

#### Using `readelf`

```shell-session
$ readelf -p .version_info /path/to/spank_qrmi.so

String dump of section '.version_info':
  [    4f]  QRMI_BUILD_VERSION:0.24.0;QRMI_GIT_HASH:0dac1793b013
```

(The exact offset shown after `[ ]` will vary depending on what else is
linked into the same `.version_info` section — see below.)

#### What each field means

| Field | Meaning |
|---|---|
| `QRMI_BUILD_VERSION` | QRMI's crate version, from `CARGO_PKG_VERSION` (i.e. the `version` field in `Cargo.toml`) at the time it was built. |
| `QRMI_GIT_HASH` | The exact git commit QRMI was built from, resolved via `git rev-parse --short=12 HEAD` in `build.rs`. Useful when the consuming project pins a moving branch (e.g. `main`) rather than a fixed release tag. |

This is especially useful when diagnosing issues caused by a version
mismatch between a deployed binary that links QRMI (such as the
[spank_qrmi](#) Slurm plugin) and the QRMI Python package used by client
workloads — you can confirm exactly which QRMI build is present without
rebuilding or adding logging.

#### If you're looking at `spank_qrmi.so` specifically

`spank_qrmi.so` embeds its own version/git-hash marker alongside QRMI's, in
the same `.version_info` section:

```shell-session
$ strings spank_qrmi.so | grep -E "SPANK_QRMI|QRMI_BUILD"
SPANK_QRMI_VERSION=0.11.0;SPANK_QRMI_GIT_HASH=0dac1793b013
QRMI_BUILD_VERSION:0.24.0;QRMI_GIT_HASH:0dac1793b013
```

The two are independent: `SPANK_QRMI_*` describes the plugin binary
itself, `QRMI_BUILD_VERSION`/`QRMI_GIT_HASH` describes the QRMI crate it was
linked against. See the spank_qrmi plugin's own [FAQ](https://github.com/qiskit-community/spank-plugins/blob/main/docs/FAQ.md) for details on the
former.

#### Notes

- If `QRMI_GIT_HASH` shows `unknown`, QRMI was most likely built from a source
  tree without a `.git` directory (e.g. an extracted release tarball, or a
  local checkout pointed at via `QRMI_ROOT` in the consuming project's
  build).
- If neither `strings` nor `readelf` show a `.version_info` section (or it
  only shows a `SPANK_QRMI_*` entry with no `QRMI_BUILD_VERSION`), the
  binary may have been built before this feature was introduced, or the
  section may have been removed by a full `strip -s` pass in the
  deployment pipeline. Re-run `strip --strip-debug` instead, or add
  `--keep-section=.version_info` to the `strip`/`objcopy` invocation, to
  preserve it.
- This works the same way regardless of whether QRMI was linked as a
  shared library (`cdylib`) or a static library (`staticlib`) — the marker
  survives static linking as long as at least one other QRMI symbol is
  referenced by the final binary.

