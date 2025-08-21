QRMI: Quantum Resource Management Interface
===========================================

This is repository with Quantum Resource Management Interface (QRMI) implementation.

QRMI ⚛️ is a vendor agnostic library to control state, run tasks and monitor behavior of quantum computational resources (qubits, QPUs, entire quantum systems, etc.).

QRMI acts like a thin middleware layer that abstracts away complexities of controling quantum resources by exposing set of simple APIs to acquire/release hardware, run tasks and monitor state of quantum resources.

QRMI is written in Rust 🦀 with Python 🐍 and C ©️ APIs exposed for ease of integration to pretty much any computational envrionment.

## 📒 Contents

- [⬇️ Installation](INSTALL.md)
- [▶️ Examples](#examples)
  - [C  ©️](#c)
  - [Python 🐍](#python)
  - [Rust 🦀](#rust)
  - [Usage in Slurm plugin for quantum resources](#qrmi-usage-in-slurm-plugin-for-quantum-resources)
- [How to Give Feedback](#how-to-give-feedback)
- [How to Cite This Work](#how-to-cite-this-work)
- [Contribution Guidelines](#contribution-guidelines)
- [References and Acknowledgements](#references-and-acknowledgements)

----

## Examples

All full examples are available in [examples folder](./examples/).


### C

This example is using QRMI with Direct Access to IBM Quantum machines.

```c++
#include "qrmi.h"

int main(int argc, char *argv[]) {
    ...

    QrmiQuantumResource *qrmi = qrmi_resource_new(argv[1], QRMI_RESOURCE_TYPE_IBM_DIRECT_ACCESS);
    ...

    QrmiReturnCode rc = qrmi_resource_metadata(qrmi, &metadata);
    ...

    QrmiResourceMetadata *metadata = NULL;
    rc = qrmi_resource_acquire(qrmi, &acquisition_token);
    ...

    QrmiPayload payload;
    payload.tag = QRMI_PAYLOAD_QISKIT_PRIMITIVE;
    payload.QISKIT_PRIMITIVE.input = (char *)input;
    payload.QISKIT_PRIMITIVE.program_id = argv[3];
    ...

    char *job_id = NULL;
    rc = qrmi_resource_task_start(qrmi, &payload, &job_id);
    ...

    QrmiTaskStatus status;
    rc = qrmi_resource_task_status(qrmi, job_id, &status);
    ...

    qrmi_resource_task_stop(qrmi, job_id);
    qrmi_string_free(job_id);
    qrmi_resource_free(qrmi);

    return EXIT_SUCCESS;

error:
    qrmi_resource_free(qrmi);
    return EXIT_FAILURE;
}
```

Full example is available [here](./examples/qrmi/c/direct_access/src/direct_access.c).

See example of QRMI C working with [IBM Qiskit Runtime Service](./examples/qrmi/c/qiskit_runtime_service/src/qiskit_runtime_service.c) or [Pasqal Cloud](./examples/qrmi/c/pasqal_cloud/src/pasqal_cloud.c).

All examples for QRMI C are available in [this folder](./examples/qrmi/c/).

### Python

This example is using QRMI with Direct Access to IBM Quantum machines.

``` python
from qrmi import QuantumResource, ResourceType, Payload, TaskStatus

# create resource handler
qrmi = QuantumResource("ibm_rensselaer", ResourceType.IBMDirectAccess)

# acquire resource
lock = qrmi.acquire()

# run task
payload = Payload.QiskitPrimitive(input=primitive_input, program_id=args.program_id)
job_id = qrmi.task_start(payload)

print(qrmi.task_result(job_id).value)

qrmi.task_stop(job_id)

# release resource
qrmi.release(lock)
```

Full example is available [here](./examples/qrmi/python/direct_access/example.py).

Python QRMI can be used to implement Qiskit primitives (Sampler and Estimator). See example of Qiskit primitives here [for IBM backends](./examples/qiskit_primitives/ibm/sampler.py) or [for Pasqal machines](./examples/qiskit_primitives/pasqal/sampler.py).

See [example](./examples/pulser_backend/pasqal/pulser_backend.py) of QRMI working with Pasqal Pulser.


### Rust

This example is using QRMI with Direct Access to IBM Quantum machines.

```rust
use qrmi::{ibm::IBMDirectAccess, models::Payload, models::TaskStatus, QuantumResource};
...

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ...
    let mut qrmi = IBMDirectAccess::new(&args.backend);

    let lock = qrmi.acquire().await?;
    ...

    let target = qrmi.target().await;
    ...

    let payload = Payload::QiskitPrimitive {
        input: contents,
        program_id: args.program_id,
    };

    let job_id = qrmi.task_start(payload).await?;
    println!("Job ID: {}", job_id);
    let one_sec = time::Duration::from_millis(1000);
    loop {
        let status = qrmi.task_status(&job_id).await?;
        println!("{:?}", status);
        if matches!(status, TaskStatus::Completed) {
            println!("{}", qrmi.task_result(&job_id).await?.value);
            break;
        } else if matches!(status, TaskStatus::Failed | TaskStatus::Cancelled) {
            break;
        }
        thread::sleep(one_sec);
    }
    let _ = qrmi.task_stop(&job_id).await;

    let _ = qrmi.release(&lock).await;
    Ok(())
}
```

Full example is available [here](./examples/qrmi/rust/direct_access/src/main.rs).

See example of QRMI Rust working with [IBM Qiskit Runtime Service](./examples/qrmi/rust/qiskit_runtime_service/src/main.rs) or [Pasqal Cloud](./examples/qrmi/rust/pasqal_cloud/src/main.rs).

All examples for QRMI C are available in [this folder](./examples/qrmi/rust/).


### QRMI usage in Slurm plugin for quantum resources

One of example of usage of QRMI in compute infrastrcture project is Slurm plugin for quantum resources.
QRMI is used in Slurm plugin to control quantum resources during lifetime of Slurm job.

See implementation and documentation of [Slurm plugin for quantum resources here](https://github.com/qiskit-community/spank-plugins).


----

### How to Give Feedback

We encourage your feedback! You can share your thoughts with us by:
- [Opening an issue](https://github.com/qiskit-community/qrmi/issues) in the repository


----

### How to Cite This Work

If you use the “Quantum Spank plugin” or "QRMI" in your research or projects, please consider citing the associated overview paper  [Quantum resources in resource management systems](https://arxiv.org/abs/2506.10052). 
This helps support the continued development and visibility of the repository. 
The BibTeX citation handle can be found in the [CITATION.bib](CITATION.bib) file.

Note that the overview paper is a work  in progress, and we expect multiple versions to be released as the project evolves.

----

### Contribution Guidelines

For information on how to contribute to this project, please take a look at our [contribution guidelines](CONTRIBUTING.md).


----

## References and Acknowledgements
1. Quantum spank plugins for Slurm https://github.com/qiskit-community/spank-plugins
1. Slurm documentation https://slurm.schedmd.com/
2. Qiskit https://www.ibm.com/quantum/qiskit
3. IBM Quantum https://www.ibm.com/quantum
4. Pasqal https://pasqal.com
5. STFC The Hartree Centre, https://www.hartree.stfc.ac.uk. This work was supported by the Hartree National Centre for Digital Innovation (HNCDI) programme.
6. Rensselaer Polytechnic Institute, Center for Computational Innovation, http://cci.rpi.edu/
