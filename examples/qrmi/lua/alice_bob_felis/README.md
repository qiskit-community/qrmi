# Alice and Bob Felis - Examples in Lua

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

See the corresponding section in [the README for the Felis Python example](../../python/alice_bob_felis/README.md#set-environment-variables)

## Generate QIR Input file

See the corresponding section in [the README for the Felis Python example](../../python/alice_bob_felis/README.md#generate-qir-input-file)

## Alternative: create the resource from a config map

Since QRMI v0.25.0, a resource can also be built from an explicit Lua table
instead of environment variables, via `qrmi.new_from_config()`:

```lua
local resource, err = qrmi.new_from_config("felis_backend_name", "alice-bob-felis", {
    QRMI_AB_FELIS_API_KEY = "...",
    QRMI_AB_FELIS_BASE_ENDPOINT = "...",
})
```

See the [0.25.0 migration guide](../../../../docs/migration/0.25.0.md) for
the full set of required/optional keys per resource type.

## How to run this example

```shell-session
lua example.lua <backend name> <qir input file>
```
