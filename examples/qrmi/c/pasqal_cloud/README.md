# Pasqal Cloud QRMI - Examples in C

## Prerequisites

* C compiler/linker, cmake and make
* [QRMI Rust library](../../../README.md)

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

## Alternative: create the resource from a config map

Since QRMI v0.25.0, a resource can also be built from an explicit config map
instead of environment variables, via `qrmi_resource_new_from_config()`.
Unlike `qrmi_resource_new()`, this does **not** fall back to environment
variables, and only reads `~/.pasqal/config` if `PASQAL_CONFIG_ROOT` or
`HOME` is set explicitly in the config map:

```c
QrmiKeyValue variables[] = {
    {(char *)"QRMI_PASQAL_CLOUD_PROJECT_ID", (char *)"your_project_id"},
    {(char *)"QRMI_PASQAL_CLOUD_AUTH_TOKEN", (char *)"your_auth_token"},
};
QrmiConfigMap config = { .variables = variables, .length = 2 };

QrmiQuantumResource *qrmi = qrmi_resource_new_from_config(
    "FRESNEL", QRMI_RESOURCE_TYPE_PASQAL_CLOUD, &config);
```

See the [0.25.0 migration guide](../../../../docs/migration/0.25.0.md) for
the full set of required/optional keys per resource type.

## Create Pulser Sequence file as input

Given a Pulser sequence `sequence`, we can convert it to a JSON string and write it to a file like this:

```python
serialized_sequence = sequence.to_abstract_repr()

with open("pulser_seq.json", "w") as f:
    f.write(serialized_sequence)
```

## How to build this example

```shell-session
$ mkdir build
$ cd build
$ cmake ..
$ make
```

## How to run this example
```shell-session
$ ./build/pasqal-cloud
pasqal-cloud <backend_name> <input file>
```
For example,
```shell-session
$ ./build/pasqal-cloud FRESNEL input.json
```
