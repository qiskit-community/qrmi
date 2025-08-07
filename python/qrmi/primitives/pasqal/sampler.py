# This code is part of Qiskit.
#
# (C) Copyright 2025 Pasqal, IBM. All Rights Reserved.
#
# This code is licensed under the Apache License, Version 2.0. You may
# obtain a copy of this license in the LICENSE.txt file in the root directory
# of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
#
# Any modifications or derivative works of this code must retain this
# copyright notice, and modified files need to carry a notice indicating
# that they have been altered from the originals.

"""Sampler V2 base class for Pasqal Cloud QRMI"""
import time
from dataclasses import dataclass, field
from typing import Iterable

from pulser import MockDevice
from qiskit import QuantumCircuit
from qiskit_pasqal_provider.providers import SamplerV2 as PasqalSamplerV2
from qiskit_pasqal_provider.providers.abstract_base import PasqalJob
from qiskit_pasqal_provider.providers.pulse_utils import (
    gen_seq,
    get_register_from_circuit,
)

from qrmi import Payload, QuantumResource, TaskStatus


@dataclass
class Options:
    """Options for :class:`~.QPPSamplerV2`"""

    default_shots: int = 1024
    """The default shots to use if none are specified in :meth:`~.run`.
    Default: 1024.
    """

    run_options: dict = field(default_factory=dict)
    """run_options: Options passed to run.
    """


class QPPSamplerV2(PasqalSamplerV2):
    """SamplerV2 base class for Pasqal QPUs
    Note: future work to make it work with BaseSamplerV2"""

    def __init__(
        self,
        qrmi: QuantumResource,
        *,
        options: dict | None = None,
    ) -> None:
        self._qrmi = qrmi
        self._options = Options(**options) if options else Options()

    def run(
        self, pubs: Iterable[QuantumCircuit], shots: int | None = None
    ) -> PasqalJob:
        # get the register from the analog gate inside QuantumCircuit
        qc = pubs[0]
        _analog_register = get_register_from_circuit(qc)

        seq = gen_seq(
            analog_register=_analog_register,
            device=MockDevice,
            circuit=qc,
        )

        sequence = seq.to_abstract_repr()
        job_runs = shots if shots else self._options.default_shots
        payload = Payload.PasqalCloud(sequence=sequence, job_runs=job_runs)
        new_task_id = self._qrmi.task_start(payload)
        results = []
        while True:
            status = self._qrmi.task_status(new_task_id)
            if status == TaskStatus.Completed:
                time.sleep(0.5)
                # Get the results
                results.append(self._qrmi.task_result(new_task_id).value)
                break
            elif status == TaskStatus.Failed:
                break
            else:
                print("Task status %s, waiting 1s" % status, flush=True)
                time.sleep(1)

        return results
