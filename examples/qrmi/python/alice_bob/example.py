import time
import json
import argparse
from dotenv import load_dotenv
from qrmi import QuantumResource, ResourceType, Payload, TaskStatus

load_dotenv()

parser = argparse.ArgumentParser(
    description="An example of a Quantum Resource from Alice and Bob's Felis API"
)
parser.add_argument("target", help="backend name")
parser.add_argument("qir_file", help="qir input file")
args = parser.parse_args()
qrmi = QuantumResource(args.target, ResourceType.AliceBobFelis)
lock = qrmi.acquire()
target_json = json.loads(qrmi.target().value)

print(qrmi.metadata())

# Prepare submission
input_params = {
    "nbShots": 50,
    "averageNbPhotons": 4
}

print(json.dumps(input_params))

with open(args.qir_file, encoding="utf-8") as f:
    qir = f.read()
    payload = Payload.AliceBobFelis(human_qir=qir, input_params=json.dumps(input_params))
    job_id = qrmi.task_start(payload)

    print(f"Task started {job_id}")

    while True:
        status = qrmi.task_status(job_id)
        if status not in [TaskStatus.Running, TaskStatus.Queued]:
            break

        time.sleep(1)

    print(f"Task ended - {qrmi.task_status(job_id)}")
    print("Results:")
    print(qrmi.task_result(job_id).value)

    qrmi.task_stop(job_id)

qrmi.release(lock)
