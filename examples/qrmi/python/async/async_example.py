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

"""An example of IBM Quantum System QRMI python-bindings (asyncio version)"""

import asyncio
import json
import argparse
from dotenv import load_dotenv
from qrmi import AsyncQuantumResource, ResourceType, Payload, TaskStatus


async def main(args: argparse.Namespace) -> None:
    """main"""
    qrmi = AsyncQuantumResource(args.backend, ResourceType.IBMQuantumSystem)
    print(qrmi)
    print(
        f"Selected resource: id={await qrmi.resource_id()} "
        f"type={str(await qrmi.resource_type())}"
    )

    print(await qrmi.is_accessible())

    lock = await qrmi.acquire()
    print(f"lock {lock}")

    target_json = json.loads((await qrmi.target()).value)
    print(json.dumps(target_json, indent=2))
    print(await qrmi.metadata())

    with open(args.input, encoding="utf-8") as f:
        primitive_input = f.read()
        payload = Payload.QiskitPrimitive(input=primitive_input, program_id=args.program_id)
        job_id = await qrmi.task_start(payload)
        print(f"Task started {job_id}")

        while True:
            status = await qrmi.task_status(job_id)
            if status not in [TaskStatus.Running, TaskStatus.Queued]:
                break

            await asyncio.sleep(1)

        print(f"Task ended - {await qrmi.task_status(job_id)}")
        print((await qrmi.task_result(job_id)).value)

        print(await qrmi.task_logs(job_id))

        await qrmi.task_stop(job_id)

    await qrmi.release(lock)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="An example of IBM Quantum System QRMI")
    parser.add_argument("backend", help="backend name")
    parser.add_argument("input", help="primitive input file")
    parser.add_argument("program_id", help="'estimator' or 'sampler'")
    parsed_args = parser.parse_args()

    load_dotenv()

    asyncio.run(main(parsed_args))
