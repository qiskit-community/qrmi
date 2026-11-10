# Python Tests in QRMI


## Directory Structure

All Python tests live under:

`python/tests/`

We separate unit and integration tests:

```
python/
├── qrmi/
│   └── ...
└── tests/
    ├── unit/
    └── integration/
```

### Unit Tests

- Fast.
- No external services.
- As a rough guide follow the source tree after `python/qrmi`.

Example:

`python/tests/unit/pulser/test_connection.py`

This allows:

- Logical grouping per vendor or framework.
- Local `conftest.py` files per submodule when needed.
- Vendor-specific utilities without cross-contamination.

### Integration Tests

- May require network access, services, or real backends.
- One file per resource type, e.g. `python/tests/integration/test_pasqal_local.py`.
- Run on pushes to `main`, weekly and on demand by `.github/workflows/e2e.yml`,
  which starts a local stand-in for each resource (Warden with its mock QPU,
  Simulated Quantum Resource).
  To add a resource type, add a test file and a job in that workflow.

Example:

`python/tests/integration/quantum_resource/test_maestro_live.py`

## Conventions

- Use `pytest`.
- File names must follow `test_*.py` to be discoverable.
- Tests should be deterministic, set local seeds as required.

Contributors are encouraged to:

- Add tests alongside their features.
- Replicate the source structure under `unit/` where appropriate.
- Introduce vendor-specific fixtures inside scoped directories.

## Maestro Local and C bindings

The tests in `unit/quantum_resource/test_maestro_local.py` run a scripted local
Unix socket server and require the rebuilt Python extension, but no Maestro daemon.
Set `QRMI_TEST_LIBRARY` to the rebuilt `libqrmi.so` to include C ABI/error/discovery
checks.

To test against a running Maestro server, opt in explicitly:

```sh
PYTHONPATH=python QRMI_TEST_MAESTRO_LIVE=1 \
    python -m pytest python/tests/integration/quantum_resource/test_maestro_live.py
```

These tests use fresh sessions, two-qubit circuits and 128 shots. They check Aer
and QCSim execution and estimation, cancellation, consumed results, discovery and
session release/replacement. Polling has a 60-second deadline, and each test's
sessions are deleted and checked for removal during cleanup, including on failure.
The default socket is `/run/maestro.sock`; `QRMI_MAESTRO_SOCKET` can select another
running server. This suite is skipped unless `QRMI_TEST_MAESTRO_LIVE=1` is set.
