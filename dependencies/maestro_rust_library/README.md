# Maestro Rust client

A client for the Maestro Local Unix socket protocol. Connections use
`/run/maestro.sock` by default; `QRMI_MAESTRO_SOCKET` optionally selects a different
socket. The setting is process-wide and should stay fixed while sessions are used.

The QRMI adapter and native request details are described in the
[Maestro native request guide](../../docs/maestro-native-api.rst).
