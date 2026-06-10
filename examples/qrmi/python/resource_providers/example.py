# This code is part of Qiskit.
#
# Copyright (C) IBM 2026
#
# This code is licensed under the Apache License, Version 2.0. You may
# obtain a copy of this license in the LICENSE.txt file in the root directory
# of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
#
# Any modifications or derivative works of this code must retain this
# copyright notice, and modified files need to carry a notice indicating
# that they have been altered from the originals.

"""Unified QRMI provider example.

Works with any supported provider type (qiskit-runtime-service,
ibm-quantum-system, etc.). The resource type is read from qrmi_config.json
— no code changes needed when switching between providers.
"""

import argparse
from dotenv import load_dotenv
from qrmi import Config, ResourceProvider

parser = argparse.ArgumentParser(
    description="Unified QRMI Provider Example"
)
parser.add_argument("config_file", help="Path to qrmi_config.json")
parser.add_argument("resource_name", help="Name of the dynamic resource definition (is_dynamic=true)")
parser.add_argument("--filters", default=None, help="Optional filter string e.g. 'num_qubits=127&name=ibm_*'")
args = parser.parse_args()

load_dotenv()

config = Config.load(args.config_file)
resource_def = config.resource_map[args.resource_name]

# Use resource_def.resource_type directly — works for any supported provider.
provider = ResourceProvider(resource_def.resource_type, resource_def.environment)
resources = provider.resources(args.filters)

for qrmi in resources:
    print(f"Selected resource: id={qrmi.resource_id()} type={str(qrmi.resource_type())}")
