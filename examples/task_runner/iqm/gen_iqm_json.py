"""generating IQM JSON from Qiskit QuantumCircuit"""
# (C) Copyright IBM 2026
# SPDX-License-Identifier: Apache-2.0
import argparse
import json
from pathlib import Path
from iqm.qiskit_iqm import IQMProvider
from qiskit import transpile, QuantumCircuit
from qiskit.providers import BackendV2

def dump_run_request(
    backend: BackendV2,
    run_input: QuantumCircuit | list[QuantumCircuit],
    shots: int = 1024,
    use_timeslot: bool = False,
    output_path: str | Path = "run_request.json",
    params_only: bool = False,
    **options,
) -> dict:
    """Serialize a RunRequest to a JSON file without submitting it to the server.

    Args:
        backend: An IQMBackend instance.
        run_input: Transpiled QuantumCircuit or list of QuantumCircuits.
        shots: Number of shots.
        output_path: Path of the JSON file to write.
        **options: Additional keyword arguments forwarded to backend.create_run_request()
                   (e.g. circuit_compilation_options).

    Returns:
        The serialized payload as a dict (for inspection).
    """
    # create_run_request() builds the RunRequest without submitting it to the server.
    run_request = backend.create_run_request(run_input, shots=shots, **options)

    # Resolve use_timeslot the same way _submit_job() does:
    #   resolved = use_timeslot or self._use_timeslot
    # self._use_timeslot is set on _IQMServerClient when the server URL ends with ":timeslot".
    backend_use_timeslot = getattr(
        getattr(getattr(backend, "client", None), "_iqm_server_client", None),
        "_use_timeslot",
        False,
    )
    resolved_use_timeslot = (
        use_timeslot or backend_use_timeslot
    )  # Resolve use_timeslot the same way _submit_job() does:
    #   resolved = use_timeslot or self._use_timeslot
    # self._use_timeslot is set on _IQMServerClient when the server URL ends with ":timeslot".
    backend_use_timeslot = getattr(
        getattr(getattr(backend, "client", None), "_iqm_server_client", None),
        "_use_timeslot",
        False,
    )
    resolved_use_timeslot = use_timeslot or backend_use_timeslot

    # Serialize using the same method as the real submission path (model_dump_json).
    json_str = run_request.model_dump_json()

    # params_only file
    output_path = Path(output_path)
    # output_path.write_text(json_str, encoding="utf-8")
    if params_only:
        pretty_json_str = json.dumps(json.loads(json_str), indent=2, ensure_ascii=False)
    else:
        full_json = {
            "iqmjson": json.loads(json_str),
            "job_type": "circuit",
            "use_timeslot": resolved_use_timeslot,
            "tag": None,
        }
        pretty_json_str = json.dumps(full_json, indent=2, ensure_ascii=False)
    output_path.write_text(pretty_json_str, encoding="utf-8")
    print(f"RunRequest written to: {output_path}  ({output_path.stat().st_size} bytes)")

    print(f"use_timeslot: {resolved_use_timeslot}")

    return json.loads(pretty_json_str)


def main():
    """main"""
    parser = argparse.ArgumentParser(
        description="A tool to generate IQM JSON from sample QuantumCircuit"
    )
    parser.add_argument("qc_alias", help="QC alias(e.g. sirius:mock)")
    parser.add_argument("base_url", help="IQM Server API endpoint")
    parser.add_argument("token", help="IQM Server API token")
    args = parser.parse_args()

    provider = IQMProvider(
        args.base_url, quantum_computer=args.qc_alias, token=args.token
    )
    backend = provider.get_backend()

    # Create a sample circuit

    # This example creates a GHZ state, replace it with your own code
    qc = QuantumCircuit(3)
    qc.h(0)
    qc.cx(0, 1)
    qc.cx(0, 2)
    qc.measure_all()
    qc.barrier()

    qc_transpiled = transpile(qc, backend)

    # dump out Submit job: circuit API payload
    payload = dump_run_request(
        backend,
        qc_transpiled,
        shots=1024,
        use_timeslot=False,
        params_only=True,
        output_path=f"iqm_json_{args.qc_alias}_params_only.json",
    )
    print(f"circuits:  {len(payload['circuits'])}")
    print(f"shots:     {payload['shots']}")
    print(f"calset_id: {payload['calibration_set_id']}")

    payload = dump_run_request(
        backend,
        qc_transpiled,
        shots=1024,
        use_timeslot=False,
        params_only=False,
        output_path=f"iqm_json_{args.qc_alias}.json",
    )

    print(f"circuits:  {len(payload['iqmjson']['circuits'])}")
    print(f"shots:     {payload['iqmjson']['shots']}")
    print(f"calset_id: {payload['iqmjson']['calibration_set_id']}")

if __name__ == "__main__":
    main()
