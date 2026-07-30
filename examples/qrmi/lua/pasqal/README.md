# Pasqal Cloud QRMI - Examples in Lua

## Prerequisites

* [QRMI C library(libqrmi.so)](../../../../README.md#standalone-c-library)
* [QRMI Lua Module(qrmi.so)](../../../../lua/README.md)

## Setup

```bash
export LUA_CPATH="</path/to/qrmi.so-dir/>?.so;;"
export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:/path/to/libqrmi.so-dir
```

Example:
```bash
export LUA_CPATH="/shared/qrmi/lua/build/?.so;;"
export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:/shared/qrmi/target/release
```


## Set environment variables

QRMI supports Pasqal Cloud configuration via environment variables. For Pasqal Cloud auth, QRMI also supports reading `~/.pasqal/config` (token or username/password). `PASQAL_CONFIG_ROOT` may point elsewhere and takes priority over `<backend_name>_PASQAL_CONFIG_ROOT`; QRMI expands `~`, `$VAR`, and `${VAR}` before appending `.pasqal/config`.  # pragma: allowlist secret

The required environment variables are listed below. This example assumes that a `.env` file is available under the current directory.


| Environment variables | Descriptions |
| ---- | ---- |
| <backend_name>_QRMI_PASQAL_CLOUD_PROJECT_ID | Pasqal Cloud Project ID to access the QPU |
| <backend_name>_QRMI_PASQAL_CLOUD_AUTH_TOKEN | Pasqal Cloud Auth Token (optional when username/password are configured) |
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

## Create Pulser Sequence file as input

Given a Pulser sequence `sequence`, we can convert it to a JSON string and write it to a file like this:

```python
serialized_sequence = sequence.to_abstract_repr()

with open("pulser_seq.json", "w") as f:
    f.write(serialized_sequence)
```

## How to run this example
```shell-session
lua example.lua <backend name> <resource type> <input file>
```
For example,
```shell-session
lua example.lua FRESNEL pasqal-cloud input.json
```
