# -*- coding: utf-8 -*-

# This code is part of Qiskit.
#
# (C) Copyright Qoro Quantum 2026
#
# This code is licensed under the Apache License, Version 2.0. You may
# obtain a copy of this license in the LICENSE.txt file in the root directory
# of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
#
# Any modifications or derivative works of this code must retain this
# copyright notice, and modified files need to carry a notice indicating
# that they have been altered from the originals.

"""Generates schema-2 native Maestro Local task runner input files."""

import argparse
import json

NATIVE_EXAMPLES = {
    "adjoint": {
        "schema_version": 2,
        "operation": "adjoint_jacobian",
        "circuit": {
            "num_qubits": 2,
            "source": "OPENQASM 2.0; qreg q[2]; ry(0.4) q[0]; cx "
            "q[0],q[1]; rx(0.9) q[1];",
        },
        "simulator": {"backend": "qcsim", "method": "statevector"},
        "observables": [
            "ZI",
            {"terms": [[0.5, "ZZ"], [2, "XI"]]},
            {"probabilities": [1]},
        ],
        "parameter_rows": [[0.4, 0.9], [0, 0]],
    },
    "checkpoint": {
        "schema_version": 2,
        "operation": "checkpoint_batch",
        "circuit": {"num_qubits": 1, "source": "OPENQASM 2.0; qreg q[1]; x q[0];"},
        "suffixes": [
            {
                "num_qubits": 1,
                "source": "OPENQASM 2.0; qreg q[1]; creg c[1]; measure " "q->c;",
            },
            {
                "num_qubits": 1,
                "source": "OPENQASM 2.0; qreg q[1]; creg c[1]; x q[0]; "
                "measure q->c;",
            },
        ],
        "simulator": {"backend": "qcsim", "method": "density_matrix"},
        "execution": {"shots": 32, "seed": 123},
        "noise": {
            "evaluation": "exact",
            "channels": [{"kind": "bit_flip", "targets": [0], "probability": 1}],
        },
    },
    "correlated-noise": {
        "schema_version": 2,
        "operation": "estimate",
        "circuit": {
            "format": "instructions",
            "num_qubits": 1,
            "source": [
                {"name": "h", "qubits": [0]},
                {"name": "id", "qubits": [0]},
                {"name": "id", "qubits": [0]},
                {"name": "id", "qubits": [0]},
            ],
        },
        "simulator": {
            "backend": "qcsim",
            "method": "matrix_product_state",
            "options": {"max_bond_dimension": 2, "optimize_circuit": False},
        },
        "observables": ["X", "Z"],
        "execution": {"seed": 123},
        "noise": {
            "mode": "combined",
            "evaluation": "trajectories",
            "seed": 23,
            "realizations": 128,
            "channels": [
                {
                    "kind": "correlated_ou",
                    "targets": [0],
                    "sigma": 0.4,
                    "alpha": 2,
                    "gate_time": 0.1,
                    "after_1q": True,
                    "after_2q": False,
                    "stationary_init": True,
                }
            ],
        },
    },
    "distributed-gpu": {
        "schema_version": 2,
        "operation": "execute",
        "circuit": {
            "num_qubits": 3,
            "source": "OPENQASM 2.0; qreg q[3]; creg c[3]; h "
            "q[0]; cx q[0],q[2]; measure q->c;",
        },
        "simulator": {
            "backend": "distributed_gpu",
            "method": "statevector",
            "options": {"precision": "double"},
            "distribution": {"devices": [0, 1], "global_qubits": [2], "backend": "ex"},
        },
        "execution": {"shots": 1024, "seed": 123},
        "noise": {
            "mode": "pauli",
            "evaluation": "trajectories",
            "realizations": 64,
            "seed": 123,
            "channels": [{"kind": "dephasing", "targets": [0, 2], "probability": 0.01}],
        },
    },
    "incremental": {
        "schema_version": 2,
        "operation": "incremental_evolve",
        "circuit": {"num_qubits": 1, "source": "OPENQASM 2.0; qreg q[1]; x q[0];"},
        "step_circuit": {
            "format": "instructions",
            "num_qubits": 1,
            "source": [
                {
                    "name": "kraus",
                    "qubits": [0],
                    "operators": [[1, 0, 0, 0.8660254037844386], [0, 0.5, 0, 0]],
                }
            ],
        },
        "steps": [0, 1, 2, 4],
        "observables": ["Z"],
        "simulator": {"backend": "qcsim", "method": "density_matrix"},
        "execution": {"seed": 123},
    },
    "instructions": {
        "schema_version": 2,
        "operation": "execute",
        "circuit": {
            "format": "instructions",
            "num_qubits": 2,
            "source": [
                {"name": "h", "qubits": [0], "noiseless": True},
                {"name": "h", "qubits": [1], "noiseless": True},
                {"name": "rzz", "qubits": [0, 1], "params": [3.141592653589793]},
                {"name": "h", "qubits": [0], "noiseless": True},
                {"name": "h", "qubits": [1], "noiseless": True},
                {"name": "pauli_channel", "qubits": [0], "probabilities": [0, 1, 0, 0]},
                {
                    "name": "measure",
                    "qubits": [0, 1],
                    "clbits": [0, 1],
                    "readout_error": {"p_meas1_prep0": 0, "p_meas0_prep1": 0},
                },
            ],
        },
        "simulator": {"backend": "qcsim", "method": "density_matrix"},
        "execution": {"shots": 32, "seed": 123},
    },
    "mirror-observables": {
        "schema_version": 2,
        "operation": "mirror_fidelity",
        "circuit": {
            "num_qubits": 3,
            "source": "OPENQASM 2.0; qreg q[3]; h q[0]; cx "
            "q[0],q[1]; cx q[1],q[2]; ry(0.3) q[2];",
        },
        "simulator": {
            "backend": "qcsim",
            "method": "matrix_product_state",
            "options": {"max_bond_dimension": 2},
        },
        "observables": ["ZII", "IZI", "IIZ", "ZZZ"],
        "execution": {"seed": 123},
    },
    "mixed-state": {
        "schema_version": 2,
        "operation": "batch",
        "requests": [
            {
                "schema_version": 2,
                "operation": "fidelity_with_statevector",
                "circuit": {
                    "format": "openqasm",
                    "num_qubits": 2,
                    "source": "OPENQASM 2.0;\n" "qreg q[2]; h q[0]; cx " "q[0],q[1];",
                },
                "simulator": {"backend": "qcsim", "method": "density_matrix"},
                "target_statevector": [
                    [0.7071067811865476, 0],
                    [0, 0],
                    [0, 0],
                    [0.7071067811865476, 0],
                ],
                "noise": {
                    "evaluation": "exact",
                    "channels": [{"kind": "t1", "targets": [0], "gamma": 0.36}],
                },
            },
            {
                "schema_version": 2,
                "operation": "density_matrix_overlap",
                "circuit": {
                    "format": "openqasm",
                    "num_qubits": 1,
                    "source": "OPENQASM 2.0;\n" "qreg q[1]; h q[0];",
                },
                "other_circuit": {
                    "format": "openqasm",
                    "num_qubits": 1,
                    "source": "OPENQASM 2.0;\n" "qreg q[1]; x q[0];",
                },
                "simulator": {"backend": "qcsim", "method": "matrix_product_operator"},
            },
        ],
    },
    "mpi-gpu": {
        "schema_version": 2,
        "operation": "execute",
        "circuit": {
            "num_qubits": 3,
            "source": "OPENQASM 2.0; qreg q[3]; creg c[3]; h q[0]; cx "
            "q[0],q[2]; measure q->c;",
        },
        "simulator": {
            "backend": "distributed_mpi_gpu",
            "method": "statevector",
            "options": {"precision": "double"},
            "distribution": {"global_qubits": [2], "backend": "ex", "mpi_p2p_bits": 0},
        },
        "execution": {"shots": 1024, "seed": 123},
        "noise": {
            "mode": "pauli",
            "evaluation": "trajectories",
            "realizations": 64,
            "seed": 123,
            "channels": [{"kind": "dephasing", "targets": [0, 2], "probability": 0.01}],
        },
        "launch": {"profile": "allocation", "ranks": 2},
    },
    "mpo-operators": {
        "schema_version": 2,
        "operation": "diagnostics",
        "circuit": {"num_qubits": 2, "source": "OPENQASM 2.0; qreg q[2]; x q[1];"},
        "simulator": {
            "backend": "qcsim",
            "method": "matrix_product_operator",
            "options": {"max_bond_dimension": 0},
        },
        "execution": {"seed": 123},
        "operators": [{"qubits": [0], "matrix": [2, 0, 0, 1], "normalize": False}],
        "move_qubits": [1],
        "diagnostics": [
            "trace",
            "element",
            "density_matrix",
            "unnormalized_density_matrix",
            "expectations_complex",
            "unnormalized_expectations",
        ],
        "row_state": "01",
        "col_state": "01",
        "observables": ["IZ", "ZI", "IZ"],
    },
    "mpo-wide": {
        "schema_version": 2,
        "operation": "state_probability",
        "circuit": {
            "num_qubits": 65,
            "source": "OPENQASM 2.0; qreg q[65]; x q[0]; x q[64];",
        },
        "simulator": {"backend": "qcsim", "method": "matrix_product_operator"},
        "execution": {"seed": 123},
        "target_state": "10000000000000000000000000000000000000000000000000000000000000001",
    },
    "mps-operators": {
        "schema_version": 2,
        "operation": "diagnostics",
        "circuit": {"num_qubits": 2, "source": "OPENQASM 2.0; qreg q[2]; "},
        "simulator": {"backend": "qcsim", "method": "matrix_product_state"},
        "execution": {"seed": 123},
        "diagnostics": ["operator_expectation"],
        "expectation_operators": [
            {"qubit": 0, "matrix": [0, 1, 1, 0]},
            {"qubit": 0, "matrix": [0, [0, -1], [0, 1], 0]},
        ],
        "move_qubits": [1],
        "maintenance": ["trim", "recanonicalize"],
    },
    "noise": {
        "schema_version": 2,
        "operation": "estimate",
        "circuit": {
            "format": "openqasm",
            "num_qubits": 1,
            "source": "OPENQASM 2.0;\nqreg q[1]; h q[0];",
        },
        "simulator": {"backend": "qcsim", "method": "density_matrix"},
        "observables": ["X", "Z"],
        "execution": {"seed": 123},
        "noise": {
            "evaluation": "exact",
            "channels": [{"kind": "t1", "targets": [0], "gamma": 0.36}],
        },
    },
    "query": {
        "schema_version": 2,
        "operation": "batch",
        "requests": [
            {
                "schema_version": 2,
                "operation": "amplitudes",
                "circuit": {
                    "num_qubits": 2,
                    "source": "OPENQASM 2.0; qreg q[2]; x q[0];",
                },
                "basis_states": [1, 2],
            },
            {
                "schema_version": 2,
                "operation": "diagnostics",
                "circuit": {
                    "format": "openqasm",
                    "num_qubits": 1,
                    "source": "OPENQASM 2.0;\nqreg q[1]; h q[0];",
                },
                "simulator": {"backend": "qcsim", "method": "density_matrix"},
                "execution": {"seed": 123},
                "noise": {
                    "evaluation": "exact",
                    "channels": [{"kind": "t1", "targets": [0], "gamma": 0.36}],
                },
                "diagnostics": ["trace", "purity", "partial_trace"],
                "keep_qubits": [0],
            },
        ],
    },
    "tensor-batch": {
        "schema_version": 2,
        "operation": "incremental_evolve",
        "circuit": {"num_qubits": 2, "source": "OPENQASM 2.0; qreg q[2]; "},
        "simulator": {"backend": "qcsim", "method": "matrix_product_state"},
        "execution": {"seed": 123},
        "step_circuit": {"num_qubits": 2, "source": "OPENQASM 2.0; qreg q[2]; x q[0];"},
        "steps": [0, 1, 2],
        "observables": ["ZI", "IZ", "ZI"],
    },
    "tensor-bulk": {
        "schema_version": 2,
        "operation": "statevector",
        "circuit": {"num_qubits": 2, "source": "OPENQASM 2.0; qreg q[2]; x q[0];"},
        "simulator": {
            "backend": "qcsim",
            "method": "matrix_product_state",
            "options": {"gate_fusion": True, "mps_sampling": "probabilities"},
        },
        "execution": {"seed": 123},
    },
    "tensor-expectations": {
        "schema_version": 2,
        "operation": "estimate",
        "circuit": {"num_qubits": 2, "source": "OPENQASM 2.0; qreg q[2]; x q[0];"},
        "simulator": {"backend": "qcsim", "method": "matrix_product_state"},
        "execution": {"seed": 123},
        "observables": ["ZI", "IZ", "ZI"],
    },
    "thermal-idle": {
        "schema_version": 2,
        "operation": "estimate",
        "circuit": {
            "format": "instructions",
            "num_qubits": 1,
            "source": [
                {"name": "h", "qubits": [0]},
                {"name": "delay", "qubits": [0], "duration": 3e-07},
            ],
        },
        "simulator": {"backend": "qcsim", "method": "density_matrix"},
        "observables": ["X", "Z"],
        "execution": {"seed": 123},
        "noise": {
            "evaluation": "exact",
            "channels": [
                {
                    "kind": "thermal_relaxation",
                    "targets": [0],
                    "duration": 1e-07,
                    "t1": 1e-06,
                    "t2": 6e-07,
                    "excited_population": 0.1,
                },
                {
                    "kind": "idle",
                    "targets": [0],
                    "t1": 1e-06,
                    "t2": 6e-07,
                    "excited_population": 0.1,
                    "detuning_hz": 250000,
                },
            ],
        },
    },
}


def main() -> None:
    """CLI entrypoint to write a native Maestro request JSON file."""
    parser = argparse.ArgumentParser(
        description="Generate a schema-2 native Maestro Local task runner input"
    )
    parser.add_argument(
        "--example",
        choices=sorted(NATIVE_EXAMPLES),
        default="noise",
        help="Named native request template to generate (default: noise)",
    )
    parser.add_argument(
        "-o",
        "--output",
        default=None,
        help="Output filename (default: native-<example>.json)",
    )
    args = parser.parse_args()
    output = args.output or f"native-{args.example}.json"
    with open(output, "w", encoding="utf-8") as output_file:
        json.dump(NATIVE_EXAMPLES[args.example], output_file, indent=2)
    print(f"Wrote {output}")


if __name__ == "__main__":
    main()
