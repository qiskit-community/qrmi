# -*- coding: utf-8 -*-

# This code is part of Qiskit.
#
# (C) Copyright 2026 Alice and Bob. All Rights Reserved.
#
# This code is licensed under the Apache License, Version 2.0. You may
# obtain a copy of this license in the LICENSE.txt file in the root directory
# of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
#
# Any modifications or derivative works of this code must retain this
# copyright notice, and modified files need to carry a notice indicating
# that they have been altered from the originals.

from qiskit_qir import to_qir_module
from qiskit_alice_bob_provider import AliceBobRemoteProvider
from qiskit import transpile
from qiskit.transpiler import PassManager
from qiskit_alice_bob_provider.plugins.state_preparation import EnsurePreparationPass
import os


class FelisQIRTranspiler:
    def __init__(self, target):
        # Authentication to Felis is required here in order to build a Backend
        # for the current resource. We do not submit the circuits via this Backend
        # but rather leverage the Backend for transpilation
        self.provider = AliceBobRemoteProvider(
            api_key=os.environ[f"QRMI_AB_FELIS_API_KEY"],
            url=os.environ[f"QRMI_AB_FELIS_BASE_ENDPOINT"],
        )
        self.backend = self.provider.get_backend(target)

    def transpile(self, circuit):
        # Borrow transpilation logic from qiskit-alice-bob-provider
        transpiled_circuit = transpile(circuit, self.backend)
        # As in AliceBobRemoteBackend.run(), run a final EnsurePreparationPass
        # after the preset pass manager has run.
        # This works around https://github.com/Qiskit/qiskit/issues/6943
        final_circuit = PassManager([EnsurePreparationPass()]).run(transpiled_circuit)
        return str(to_qir_module(final_circuit)[0])
