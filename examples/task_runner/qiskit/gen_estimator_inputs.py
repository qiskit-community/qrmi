# -*- coding: utf-8 -*-

# (C) Copyright 2025 IBM. All Rights Reserved.
#
# This code is licensed under the Apache License, Version 2.0. You may
# obtain a copy of this license in the LICENSE.txt file in the root directory
# of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
#
# Any modifications or derivative works of this code must retain this
# copyright notice, and modified files need to carry a notice indicating
# that they have been altered from the originals.

"""generating input files for estimatorV2"""

# pylint: disable=invalid-name, duplicate-code
import sys
import json
import argparse
import requests

from ibm_cloud_sdk_core.authenticators import IAMAuthenticator

from qiskit_ibm_runtime.utils.backend_converter import convert_to_target
from qiskit_ibm_runtime.utils import RuntimeEncoder
from qiskit_ibm_runtime.models import BackendProperties, BackendConfiguration
from qiskit import qasm3
from qiskit.circuit.library import QAOAAnsatz
from qiskit.quantum_info import SparsePauliOp
from qiskit.transpiler.preset_passmanagers import generate_preset_pass_manager
from qiskit.primitives.containers.estimator_pub import EstimatorPub

parser = argparse.ArgumentParser(
    description="A tool to generate EstimatorV2 input for testing"
)
parser.add_argument("backend", help="Backend name")
parser.add_argument("base_url", help="API endpoint")
parser.add_argument("apikey", help="IAM API key")
parser.add_argument("crn", help="Service CRN of your instance")
parser.add_argument(
    "--iam_url", help="IAM endpoint", default="https://iam.cloud.ibm.com"
)
args = parser.parse_args()

# Use IAM based authentication
token_manager = IAMAuthenticator(apikey=args.apikey, url=args.iam_url).token_manager
headers = {
    "Authorization": f"Bearer {token_manager.get_token()}",
    "Service-CRN": args.crn,
}
print(json.dumps(headers, indent=2))

backends_url = f"{args.base_url}/v1/backends"
backends_response = requests.get(backends_url, headers=headers, timeout=10)
if backends_response.status_code == 200:
    print(json.dumps(backends_response.json(), indent=4))
else:
    print(backends_response.__dict__)

backend_config_url = f"{args.base_url}/v1/backends/{args.backend}/configuration"
backend_config_resp = requests.get(backend_config_url, headers=headers, timeout=10)
if backend_config_resp.status_code == 200:
    backend_config_json = backend_config_resp.json()
    print(json.dumps(backend_config_json, indent=4))
    backend_config = BackendConfiguration.from_dict(backend_config_json)
    print(backend_config)
else:
    print(backend_config_resp.__dict__)
    sys.exit()

backend_props_url = f"{args.base_url}/v1/backends/{args.backend}/properties"
backend_props_resp = requests.get(backend_props_url, headers=headers, timeout=10)
if backend_props_resp.status_code == 200:
    backend_props_json = backend_props_resp.json()
    print(json.dumps(backend_props_json, indent=4))
    backend_props = BackendProperties.from_dict(backend_props_json)
    print(backend_props)
else:
    print(backend_props_resp.__dict__)
    sys.exit()

# Generate transpiler target from backend configuration & properties
target = convert_to_target(backend_config, backend_props)


# Create a circuit and an observable
# You need at least one circuit and one observable as inputs to the Estimator primitive.
entanglement = [tuple(edge) for edge in target.build_coupling_map().get_edges()]
observable = SparsePauliOp.from_sparse_list(
    [("ZZ", [i, j], 0.5) for i, j in entanglement],
    num_qubits=target.num_qubits,
)

circuit = QAOAAnsatz(observable, reps=2)
# the circuit is parametrized, so we will define the parameter values for execution
param_values = [0.1, 0.2, 0.3, 0.4]

print(f">>> Observable: {observable.paulis}")

pm = generate_preset_pass_manager(
    optimization_level=1,
    target=target,
)
# Convert to an ISA circuit and layout-mapped observables.
isa_circuit = pm.run(circuit)
isa_observable = observable.apply_layout(isa_circuit.layout)
print(f">>> Circuit ops (ISA): {isa_circuit.count_ops()}")

# Generate QASM3 instructions
pub = EstimatorPub.coerce((isa_circuit, isa_observable, param_values))
qasm3_str = qasm3.dumps(
    pub.circuit,
    disable_constants=True,
    allow_aliasing=True,
    experimental=qasm3.ExperimentalFeatures.SWITCH_CASE_V1,
)

observables = pub.observables.tolist()
param_array = pub.parameter_values.as_array(pub.circuit.parameters).tolist()

# Generates JSON representation of estimator job
input_json = {
    "pubs": [(qasm3_str, observables, param_array)],
    "version": 2,
    "support_qiskit": False,
    "resilience_level": 1,
    "options": {
        "default_shots": 5000,
    },
}

def dump(json_data: dict, filename: str) -> None:
    """Write json data to the specified file

    Args:
        json_data(dict): JSON data
        filename(str): output filename
    """
    print(json.dumps(json_data, cls=RuntimeEncoder, indent=2))
    with open(filename, "w", encoding="utf-8") as primitive_input_file:
        json.dump(json_data, primitive_input_file, cls=RuntimeEncoder, indent=2)

dump(
    {"parameters": input_json, "program_id": "estimator"},
    f"estimator_input_{args.backend}.json",
)
dump(input_json, f"estimator_input_{args.backend}_params_only.json")
print("done")
