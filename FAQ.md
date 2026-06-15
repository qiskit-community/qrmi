# Frequently Asked Questions

## Table of Contents

1. [General Question](#general-questions)
2. [Job Execution Errors](#job-execution-errors)
    1. [I get an error `spank_qrmi_c, failed to acquire resource: ibm_brisbane`](i-get-an-error-spank_qrmi_c-failed-to-acquire-resource-ibm_brisbane)

## Job Execution Errors

### I get an error `spank_qrmi_c, failed to acquire resource: ibm_brisbane`

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
- Use 'Batch' execution mode
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
