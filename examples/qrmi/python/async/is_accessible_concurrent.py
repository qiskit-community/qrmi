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

"""Check accessibility of multiple IBM Quantum System backends concurrently."""

import asyncio
import argparse
from dotenv import load_dotenv
from qrmi import AsyncQuantumResource, ResourceType


async def check_one(backend: str) -> tuple[str, bool]:
    """Check accessibility of one backend"""
    qrmi = AsyncQuantumResource(backend, ResourceType.IBMQuantumSystem)
    accessible = await qrmi.is_accessible()
    return backend, accessible


async def main(backends: list[str]) -> None:
    """main"""

    # Each `check_one(...)` call creates its own AsyncQuantumResource, so
    # the awaits below run concurrently on QRMI's shared tokio runtime —
    # this takes roughly as long as the single slowest backend check, not
    # the sum of all of them.
    results = await asyncio.gather(*(check_one(b) for b in backends))

    for backend, accessible in results:
        print(f"{backend}: is_accessible={accessible}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        description="Check accessibility of multiple backends concurrently"
    )
    parser.add_argument("backends", nargs="+", help="one or more backend names")
    parsed_args = parser.parse_args()

    load_dotenv()

    asyncio.run(main(parsed_args.backends))
