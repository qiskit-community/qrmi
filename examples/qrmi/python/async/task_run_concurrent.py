#
# (C) Copyright 2026 IBM. All Rights Reserved.
#
# This code is licensed under the Apache License, Version 2.0. You may
# obtain a copy of this license in the LICENSE.txt file in the root directory
# of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
#
# Any modifications or derivative works of this code must retain this
# copyright notice, and modified files need to carry a notice indicating
# that they have been altered from the originals.

"""Submit a primitive input to multiple IBM Quantum System backends
concurrently, one input file and one program id (estimator/sampler) per
backend, and collect all results once every job has finished.
"""

import asyncio
import argparse
from dotenv import load_dotenv
from qrmi import AsyncQuantumResource, ResourceType, Payload, TaskStatus


async def run_on_backend(backend: str, primitive_input: str, program_id: str) -> str:
    """Acquire `backend`, run one task on it, and return the result JSON.

    This whole function is one coroutine, so multiple calls to it (one per
    backend) can be scheduled together with `asyncio.gather`. The `await`s
    inside it (task_start / task_status / task_result / ...) yield control
    back to the event loop, letting the *other* backends' coroutines make
    progress while this one is, say, waiting on a queued job.
    """
    qrmi = AsyncQuantumResource(backend, ResourceType.IBMQuantumSystem)

    lock = await qrmi.acquire()
    try:
        payload = Payload.QiskitPrimitive(input=primitive_input, program_id=program_id)
        job_id = await qrmi.task_start(payload)
        print(f"[{backend}] task started: {job_id}")

        while True:
            status = await qrmi.task_status(job_id)
            if status not in [TaskStatus.Running, TaskStatus.Queued]:
                break
            await asyncio.sleep(1)

        print(f"[{backend}] task ended: {status}")
        result = await qrmi.task_result(job_id)
        await qrmi.task_stop(job_id)
        return result.value
    finally:
        # Runs even if task_start/task_status/task_result raised, so a
        # failure on one backend doesn't leave its lock held forever.
        await qrmi.release(lock)


async def main(backends: list[str], input_paths: list[str], program_ids: list[str]) -> None:
    """main"""
    primitive_inputs = []
    for path in input_paths:
        with open(path, encoding="utf-8") as f:
            primitive_inputs.append(f.read())

    # `return_exceptions=True` so one backend's failure doesn't cancel the
    # others' in-flight work; each entry in `results` is either the result
    # JSON string or the exception that was raised for that backend.
    results = await asyncio.gather(
        *(
            run_on_backend(b, primitive_input, program_id)
            for b, primitive_input, program_id in zip(backends, primitive_inputs, program_ids)
        ),
        return_exceptions=True,
    )

    for backend, result in zip(backends, results):
        if isinstance(result, Exception):
            print(f"[{backend}] FAILED: {result!r}")
        else:
            print(f"[{backend}] result: {result}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        description="Run a primitive on multiple backends concurrently"
    )
    parser.add_argument(
        "--backend",
        action="append",
        required=True,
        dest="backends",
        metavar="BACKEND",
        help="backend name; repeat once per backend, paired by order with --input/--program-id",
    )
    parser.add_argument(
        "--input",
        action="append",
        required=True,
        dest="inputs",
        metavar="FILE",
        help="primitive input file; repeat once per backend, paired by order with --backend/--program-id",
    )
    parser.add_argument(
        "--program-id",
        action="append",
        required=True,
        dest="program_ids",
        metavar="'estimator'|'sampler'",
        help="program id; repeat once per backend, paired by order with --backend/--input",
    )
    parsed_args = parser.parse_args()

    counts = {
        "--backend": len(parsed_args.backends),
        "--input": len(parsed_args.inputs),
        "--program-id": len(parsed_args.program_ids),
    }
    if len(set(counts.values())) != 1:
        parser.error(
            "each --backend needs exactly one matching --input and --program-id, "
            f"in order, but got: {counts}"
        )

    load_dotenv()

    asyncio.run(main(parsed_args.backends, parsed_args.inputs, parsed_args.program_ids))
