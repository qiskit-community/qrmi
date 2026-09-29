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

## Conventions

- Use `pytest`.
- File names must follow `test_*.py` to be discoverable.
- Tests should be deterministic, set local seeds as required.

Contributors are encouraged to:

- Add tests alongside their features.
- Replicate the source structure under `unit/` where appropriate.
- Introduce vendor-specific fixtures inside scoped directories.
