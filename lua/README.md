# QRMI Lua Binding

A Lua binding for the QRMI C API. Implemented using the Lua C API approach for standard Lua (PUC-Rio Lua 5.1-5.4).

## Usage from Lua

```lua
local qrmi = require("qrmi")

local resource, err = qrmi.new("ibm_kingston", "qiskit-runtime-service")
if not resource then error(err) end

local accessible = resource:is_accessible()

local token = resource:acquire()

local payload_file = io.open("task_payload.txt", "r")
local input_json = payload_file:read("*a")
payload_file:close()

local task_id = resource:task_start({
    qiskit_primitive = {
        program_id = "estimator",
        input = input_json,
    }
})

local status = resource:task_status(task_id)   -- "queued"/"running"/"completed"/"failed"/"cancelled"
local result = resource:task_result(task_id)
local logs   = resource:task_logs(task_id)

resource:release()
resource:free()   -- explicit release (also happens automatically via __gc if omitted)
```

Valid strings for `resource_type` (matching qrmi_config_resource_type_to_str()'s
canonical, hyphen-separated names):
`ibm-quantum-system` / `qiskit-runtime-service` / `pasqal-cloud` / `pasqal-local` / `alice-bob-felis` / `iqm-server`

## `qrmi.config` — independent from `qrmi.resource`

`qrmi.config` wraps `QrmiConfig` (a `qrmi_config.json` file) and has a
completely separate lifecycle from `qrmi.resource`; the two are unrelated.

```lua
local config, err = qrmi.load_config("/etc/slurm/qrmi_config.json")
if not config then error(err) end

local def, err2 = config:resource_def("ibm_kingston")
-- def = {
--   name = "ibm_kingston",
--   type = "ibm-quantum-system",
--   is_dynamic = false,
--   environments = { QRMI_IBM_QUANTUM_TOKEN = "...", QRMI_IBM_QUANTUM_CHANNEL = "..." },
-- }

config:free()   -- explicit release (also happens automatically via __gc if omitted)
```

## Prerequisites

* `lua' and 'lua-devel` Linux packages
* [QRMI Standalone C library & header](../README.md)

## Building

Assuming the `qrmi.h` and `libqrmi.so` live in the same directory
(`<QRMI_ROOT>`, e.g. `/path/to/qrmi`):

```bash
gcc -shared -fPIC -O2 $(pkg-config --cflags lua5.4) \
    -I/path/to/qrmi \
    -o qrmi.so lua_qrmi.c \
    -L/path/to/qrmi -lqrmi \
    $(pkg-config --libs lua5.4) \
    -Wl,-rpath,/path/to/qrmi
```

- `-I/path/to/qrmi` — lets the compiler find `qrmi.h`
- `-L/path/to/qrmi -lqrmi` — links against `libqrmi.so`
- `-Wl,-rpath,/path/to/qrmi` — bakes that directory into `qrmi.so`'s RUNPATH, so `LD_LIBRARY_PATH` doesn't need to be set at runtime (verify with `readelf -d qrmi.so`)

## Design notes

1. **Converting QrmiReturnCode into Lua's (value, err) pattern**
   Every real API call returns a `QrmiReturnCode`, so on failure `qrmi_get_last_error()`
   is called to fetch a detailed message, returned to Lua as `(nil, errmsg)`.
   On success, the out-parameter's value is pushed directly.

2. **String ownership management**
   Any `char*` heap-allocated on the Rust side (via cbindgen) is freed with `qrmi_string_free()`.
   This implementation frees it immediately after copying it into Lua (`lua_pushstring`), so nothing leaks.

3. **Resource lifecycle and GC**
   `QrmiQuantumResource*` is an opaque pointer. The userdata holds exactly one,
   and the `__gc` metamethod ensures it's automatically `release`d and `free`d
   even if the caller forgets to do so explicitly.

4. **Building the QrmiPayload (tagged union)**
   Currently only `QRMI_PAYLOAD_QISKIT_PRIMITIVE` is supported. Pasqal Cloud / Alice&Bob Felis / IQM Server
   can be added following the same pattern (`payload.tag = QRMI_PAYLOAD_XXX; payload.XXX.field = ...;`).

## Not yet implemented (room for extension)

- `QrmiResourceProvider` / `qrmi_provider_new()` — resource discovery / least-busy selection
- Redirecting logs to Lua via `qrmi_log_callback_set()`

## Building with CMake

A `CMakeLists.txt` is included. It always links against the real `libqrmi.so`.

Expected layout (header and library in the same directory):
```
<QRMI_ROOT>/qrmi.h
<QRMI_ROOT>/libqrmi.so
```

```bash
mkdir build && cd build
cmake -DQRMI_ROOT=/path/to/qrmi/install ..
cmake --build .
```

To specify paths individually:
```bash
cmake -DQRMI_INCLUDE_DIR=/path/to/include -DQRMI_LIBRARY=/path/to/libqrmi.so ..
```

The directory containing `libqrmi.so` is automatically baked into the built
artifact's RUNPATH, so there's no need to set `LD_LIBRARY_PATH` at runtime
(verifiable with `readelf -d qrmi.so`).
