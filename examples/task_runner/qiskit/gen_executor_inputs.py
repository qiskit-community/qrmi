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

"""generating input files for Executor"""

# pylint: disable=invalid-name, duplicate-code
import sys
import json
import argparse
import requests

from ibm_cloud_sdk_core.authenticators import IAMAuthenticator

from qiskit_ibm_runtime.quantum_program.params_converters import (
    QUANTUM_PROGRAM_PARAMS_CONVERTERS,
)
from qiskit_ibm_runtime.utils.backend_converter import convert_to_target

try:
    # qiskit-ibm-runtime >= 0.46
    from qiskit_ibm_runtime import RuntimeEncoder
except ModuleNotFoundError:
    # qiskit_ibm_runtime < 0.46
    from qiskit_ibm_runtime.utils import RuntimeEncoder
from qiskit_ibm_runtime.models import BackendProperties, BackendConfiguration
from qiskit_ibm_runtime.quantum_program import QuantumProgram
from qiskit_ibm_runtime.options_models.executor import ExecutorOptions
import numpy as np
from samplomatic import build
from samplomatic.transpiler import generate_boxing_pass_manager
from qiskit.transpiler.preset_passmanagers import generate_preset_pass_manager
from qiskit.circuit import Parameter, QuantumCircuit

DEFAULT_SCHEMA_VERSION = "v2.0"

parser = argparse.ArgumentParser(
    description="A tool to generate Executor input for testing"
)
parser.add_argument("backend", help="Backend name")
parser.add_argument("base_url", help="API endpoint")
parser.add_argument("apikey", help="IAM API key")
parser.add_argument(
    "instance", help="Service CRN of your instance - starting with 'crn:v1:'"
)
parser.add_argument(
    "--iam_url", help="IAM endpoint", default="https://iam.cloud.ibm.com"
)
parser.add_argument(
    "--schema_version",
    help=f"Executor schema version. default: {DEFAULT_SCHEMA_VERSION}",
    default=DEFAULT_SCHEMA_VERSION,
)
args = parser.parse_args()

try:
    converter = QUANTUM_PROGRAM_PARAMS_CONVERTERS[args.schema_version]
except KeyError as err:
    raise ValueError(
        f"Invalid schema version '{args.schema_version}'. "
        f"Supported versions: {list(QUANTUM_PROGRAM_PARAMS_CONVERTERS.keys())}"
    ) from err

# Use IAM based authentication
token_manager = IAMAuthenticator(apikey=args.apikey, url=args.iam_url).token_manager
headers = {
    "Authorization": f"Bearer {token_manager.get_token()}",
    "Service-CRN": args.instance,
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

# Generate the circuit
circuit = QuantumCircuit(3)
circuit.h(0)
circuit.h(1)
circuit.cz(0, 1)
circuit.h(1)
circuit.h(2)
circuit.cz(1, 2)
circuit.h(2)
circuit.rz(Parameter("theta"), 0)
circuit.rz(Parameter("phi"), 1)
circuit.rz(Parameter("lam"), 2)
circuit.measure_all()

# Transpile the circuit to ISA
preset_pass_manager = generate_preset_pass_manager(target=target, optimization_level=3)
isa_circuit = preset_pass_manager.run(circuit)

boxing_pm = generate_boxing_pass_manager(
    # Add gate twirling
    enable_gates=True,
    # Add measurement twirling
    enable_measures=True,
)

boxed_circuit = boxing_pm.run(isa_circuit)

# Build the template circuit and the samplex
template_circuit, samplex = build(boxed_circuit)

# Generate a quantum program
program = QuantumProgram(shots=1024)

# Append the circuit and the parameter values to the program
program.append_circuit_item(
    isa_circuit,
    circuit_arguments=np.random.rand(10, 3),  # 10 sets of parameter values
)

# Append the template circuit and samplex as a samplex item
program.append_samplex_item(
    template_circuit,
    samplex=samplex,
    samplex_arguments={
        "parameter_values": np.random.rand(10, 3),  # 10 sets of parameter values
    },
    shape=(2, 14, 10),
)


options = ExecutorOptions()
params = converter.encoder(program, options)
input_json = params.model_dump(mode="json")
print(json.dumps(input_json, indent=2))

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
    {"parameters": input_json, "program_id": "executor"},
    f"executor_input_{args.backend}_{args.schema_version}.json",
)
dump(input_json, f"executor_input_{args.backend}_{args.schema_version}_params_only.json")

print("done")
