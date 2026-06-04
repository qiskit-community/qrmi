# Run the Alice & Bob Felis example using qrmi from PyPI.
#
# Environment variables required at runtime:
#   QRMI_AB_FELIS_API_KEY
#   QRMI_AB_FELIS_BASE_ENDPOINT
#
# Usage:
#   docker build -f docker/alice-bob-felis-example/Dockerfile -t qrmi-ab-felis-example .
#   docker run --rm \
#     -e QRMI_AB_FELIS_API_KEY=<key> \
#     -e QRMI_AB_FELIS_BASE_ENDPOINT=https://api-staging.alice-bob.com \
#     qrmi-ab-felis-example

FROM python:3.12-slim

WORKDIR /app

RUN pip install --no-cache-dir qrmi

# Copy the example script and circuit
COPY examples/qrmi/python/alice_bob_felis/example.py ./example.py
COPY generated_circuit.ll ./generated_circuit.ll

ENTRYPOINT ["python", "example.py", "ab_emu_1q_lescanne_2020", "generated_circuit.ll"]
