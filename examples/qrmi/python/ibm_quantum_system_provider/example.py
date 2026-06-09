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

"""An example of QRMI Provider for IBM Quantum System python-bindings"""

import argparse
from dotenv import load_dotenv
from qrmi import Config, ResourceProvider

parser = argparse.ArgumentParser(
    description="An example of QRMI Provider for IBM Quantum System"
)
parser.add_argument("config_file", help="Path to qrmi_config.json")
parser.add_argument("resource_name", help="Name of the dynamic resource definition")
parser.add_argument("--filters", default=None, help="Optional filter string e.g. 'num_qubits=27&name=test_*'")
args = parser.parse_args()

load_dotenv()

config = Config.load(args.config_file)
resource_def = config.resource_map[args.resource_name]

provider = ResourceProvider(resource_def)
resources = provider.resources(args.filters)
for qrmi in resources:
    print(f"Selected resource: id={qrmi.resource_id()} type={str(qrmi.resource_type())}")
