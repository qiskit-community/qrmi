# -*- coding: utf-8 -*-

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

"""Estimator V2 base class for IBM QRMI"""
import json
from typing import Union
from dataclasses import dataclass, field
from collections.abc import Iterable
from qiskit import qasm3
from qiskit.primitives.base import BaseEstimatorV2
from qiskit.primitives.containers.estimator_pub import EstimatorPub, EstimatorPubLike

from qrmi import QuantumResource, Payload

from .runtime_job_v2 import RuntimeJobV2


@dataclass
class Options:
    """Options for :class:`~.QRMIBaseEstimatorV2`."""

    default_precision: float = 0.015625
    """The default precision to use if none are specified in :meth:`~run`.
    Default: 0.015625 (1 / sqrt(4096)).
    """

    run_options: dict = field(default_factory=dict)
    """run_options: Options passed to run."""


class QRMIBaseEstimatorV2(BaseEstimatorV2):
    """Estimator V2 base class for IBM QRMI"""

    def __init__(
        self,
        qrmi: QuantumResource,
        *,
        options: dict | None = None,
    ) -> None:
        self._qrmi = qrmi
        self._options = Options(**options) if options else Options()

    def run(self, pubs: Iterable[EstimatorPubLike], *, precision: float | None = None):
        if precision is None:
            precision = self._options.default_precision

        # for each Pub (Primitive Unified Bloc)
        dict_pubs = []
        for pub in pubs:
            # Coerce a EstimatorPubLike object into a EstimatorPub instance.
            coerced_pub = EstimatorPub.coerce(pub, precision)
            # Generate OpenQASM3 string which can be consumed by IBM Quantum APIs
            qasm3_str = qasm3.dumps(
                coerced_pub.circuit,
                disable_constants=True,
                allow_aliasing=True,
                experimental=qasm3.ExperimentalFeatures.SWITCH_CASE_V1,
            )

            observables = coerced_pub.observables.tolist()
            param_array = coerced_pub.parameter_values.as_array(
                coerced_pub.circuit.parameters
            ).tolist()

            if len(coerced_pub.circuit.parameters) == 0:
                if coerced_pub.precision is None:
                    dict_pubs.append((qasm3_str, observables))
                else:
                    dict_pubs.append(
                        (qasm3_str, observables, param_array, coerced_pub.precision)
                    )
            else:
                if coerced_pub.precision is None:
                    dict_pubs.append((qasm3_str, observables, param_array))
                else:
                    dict_pubs.append(
                        (qasm3_str, observables, param_array, coerced_pub.precision)
                    )

        # Create EstimatorV2 input
        # https://github.com/Qiskit/ibm-quantum-schemas/blob/main/schemas/estimator_v2_schema.json
        input_json = {
            "pubs": dict_pubs,
            "options": self._options.run_options,
            "version": 2,
            "support_qiskit": True,
        }

        if precision:
            input_json["precision"] = precision

        payload = Payload.QiskitPrimitive(
            input=json.dumps(input_json), program_id="estimator"
        )
        job_id = self._qrmi.task_start(payload)
        return RuntimeJobV2(self._qrmi, job_id, delete_job=True)
