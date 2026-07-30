# QRMI Lua Module

A Lua binding for the QRMI C API. Implemented using the Lua C API approach for standard Lua (PUC-Rio Lua 5.1-5.4).


## Prerequisites

* Standard Lua (PUC-Rio Lua 5.1-5.4)
* [QRMI Standalone C library & header](../README.md#standalone-c-library)

## Installation of Lua

### RHEL / Clone OS (Rocky Linux, AlmaLinux)

For Rocky Linux 9 and AlmaLinux 8, you need to enable the additional repository (CRB or PowerTools) to install the development packages (`-devel`).

#### Rocky Linux 9
```bash
sudo dnf config-manager --set-enabled crb
sudo dnf install lua lua-devel
```

#### AlmaLinux 8
```bash
sudo dnf config-manager --set-enabled powertools
sudo dnf install lua lua-devel
```

### Debian / Ubuntu

On Debian and Ubuntu, development packages use the `-dev` suffix instead of `-devel`. You can specify the Lua version (e.g., `5.4`) during installation.

```bash
sudo apt update
sudo apt install lua5.4 liblua5.4-dev
```
> [!NOTE]
> You can replace `5.4` with other versions like `5.3` or `5.1` depending on your requirements.

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


## API Reference

[API Reference](./API_REFERENCE.md)

## API example

[This directory](../examples/qrmi/lua) contains API examples.
