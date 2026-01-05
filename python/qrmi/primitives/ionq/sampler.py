# This code is part of Qiskit.
#
# (C) Copyright 2025 IBM. All Rights Reserved.
#
# This code is licensed under the Apache License, Version 2.0. You may
# obtain a copy of this license in the LICENSE.txt file in the root directory
# of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
#
# Any modifications or derivative works of this code must retain this
# copyright notice, and modified files need to carry a notice indicating
# that they have been altered from the originals.

"""Sampler V2 base class for IonQ Cloud QRMI"""

import os
import time

from dataclasses import dataclass, field
from qiskit import transpile
from qiskit.circuit import QuantumCircuit
from qiskit_ionq.ionq_job import IonQJob
from qrmi import Payload, QuantumResource, ResourceType, TaskStatus
from typing import Iterable

from qrmi import Payload, QuantumResource, TaskStatus


@dataclass
class Options:
    """Options for :class:`~.IonQSamplerV2`"""

    default_shots: int = 1024
    """The default shots to use if none are specified in :meth:`~.run`.
    Default: 1024.
    """

    run_options: dict = field(default_factory=dict)
    """run_options: Options passed to run.
    """


class IonQSamplerV2:
    """Sampler V2 base class for IonQ QPUs"""

    def __init__(
        self,
        qrmi: QuantumResource,
        *,
        options: dict | None = None,
    ) -> None:
        self._qrmi = qrmi
        self._options = Options(**options) if options else Options()

    def run(self, pubs: Iterable[QuantumCircuit], shots: int | None = None) -> IonQJob:

        if shots is None:
            shots = self._options.default_shots

        # Work in progress ..

        # payload = Payload.IonQCloud(input=input, target=target, shots=shots)
        # new_task_id = self._qrmi.task_start(payload)
        # results = []
        # while True:
        #     status = self._qrmi.task_status(new_task_id)
        #     if status == TaskStatus.Completed:
        #         time.sleep(0.5)
        #         # Get the results
        #         results.append(self._qrmi.task_result(new_task_id).value)
        #         break
        #     elif status == TaskStatus.Failed:
        #         break
        #     else:
        #         print("Task status %s, waiting 1s" % status, flush=True)
        #         time.sleep(1)

        # return results


def main():
    qrmi = QuantumResource("simulator", ResourceType.IonQCloud)
    sampler = IonQSamplerV2(qrmi)
    sampler.run([], shots=100)


if __name__ == "__main__":
    main()
