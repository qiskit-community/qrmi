# Direct Access API for Rust Examples

## Prerequisites
* Rust 1.81.0 or above


## Build examples
```shell-session
. ~/.cargo/env
cargo clean
cargo build --release
```

## Examples

### backend
Invoke Backend APIs.

Source: [backend/src/main.rs](./backend/src/main.rs)

Usage:
```bash
Usage: backend
```

Example:
```bash
./target/release/backend
```

### cancel_job
Cancel a job.

Source: [cancel_job/src/main.rs](./cancel_job/src/main.rs)

Usage:
```bash
Usage: cancel_job [OPTIONS] --job-id <JOB_ID>

Options:
  -j, --job-id <JOB_ID>  ID of job to be cancelled
  -d, --delete-job       true if delete a job after cancelled
  -h, --help             Print help
  -V, --version          Print version
```

Example:
```bash
./target/release/cancel_job -j eec2b1ab-91a6-4ad7-aeff-146de42b21cd
```

### delete_job
Delete a job.

Source: [delete_job/src/main.rs](./delete_job/src/main.rs)

Usage:
```bash
Usage: delete_job --job-id <JOB_ID>

Options:
  -j, --job-id <JOB_ID>  ID of job to be cancelled
  -h, --help             Print help
  -V, --version          Print version
```

Example:
```bash
./target/release/delete_job -j eec2b1ab-91a6-4ad7-aeff-146de42b21cd
```

### job_details
Access job details

Source: [job_details/src/main.rs](./job_details/src/main.rs)

Usage:
```bash
Usage: job_details --job-id <JOB_ID>

Options:
  -j, --job-id <JOB_ID>  ID of job to be cancelled
  -h, --help             Print help
  -V, --version          Print version
```

Example:
```bash
./target/release/job_details -j eec2b1ab-91a6-4ad7-aeff-146de42b21cd
```

### list_jobs
List jobs.

Source: [list_jobs/src/main.rs](./list_jobs/src/main.rs)

Usage:
```bash
Usage: list_jobs
```

Example:
```bash
./target/release/list_jobs
```

### run_job
Run a job.

Source: [run_job/src/main.rs](./run_job/src/main.rs)

Usage:
```bash
Usage: run_job --backend-name <BACKEND_NAME> --job <JOB> --program-id <PROGRAM_ID> --log-level <LOG_LEVEL>

Options:
  -b, --backend-name <BACKEND_NAME>  backend name
  -j, --job <JOB>                    job input file
  -p, --program-id <PROGRAM_ID>      program id
  -l, --log-level <LOG_LEVEL>        logging level
  -h, --help                         Print help
  -V, --version                      Print version
```

Example:
```bash
./target/release/run_job -b fake_brisbane -j ../../utils/sampler_input.json -p sampler -l info
```

### run_primitive
Invoke a Qiskit Runtime primitive.

Source: [run_primitive/src/main.rs](./run_primitive/src/main.rs)

Usage:
```bash
Usage: run_primitive --backend-name <BACKEND_NAME> --job <JOB> --program-id <PROGRAM_ID> --log-level <LOG_LEVEL>

Options:
  -b, --backend-name <BACKEND_NAME>  backend name
  -j, --job <JOB>                    job input file
  -p, --program-id <PROGRAM_ID>      program id
  -l, --log-level <LOG_LEVEL>        logging level
  -h, --help                         Print help
  -V, --version                      Print version
```

Example:
```bash
./target/release/run_primitive -b fake_cairo -j ../../utils/estimator_input.json -p estimator -l warning
```

### version
Show service version.

Source: [version/src/main.rs](./version/src/main.rs)

Usage:
```bash
Usage: version
```

Example:
```bash
./target/release/version
```
