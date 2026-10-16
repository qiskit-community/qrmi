FROM rust:1.91-slim-bookworm AS builder
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential pkg-config libssl-dev python3-dev python3-venv \
    && rm -rf /var/lib/apt/lists/*
RUN python3 -m venv /build-venv \
    && /build-venv/bin/pip install "maturin>=1.8,<2"
WORKDIR /build
COPY Cargo.toml Cargo.lock rust-toolchain.toml pyproject.toml ./
COPY build.rs cbindgen.toml LICENSE.txt README.md ./
COPY dependencies/ dependencies/
COPY src/ src/
COPY python/ python/
COPY operator/Cargo.toml operator/Cargo.toml
COPY operator/src/ operator/src/
RUN /build-venv/bin/maturin build --release --locked \
    --interpreter /usr/bin/python3 --out /wheels

FROM python:3.11-slim-bookworm
COPY --from=builder /wheels/ /wheels/
RUN pip install --no-cache-dir /wheels/*.whl \
    "qiskit==2.5.2" "qiskit-ibm-runtime==0.50.0" \
    qiskit-qasm3-import && rm -rf /wheels
WORKDIR /app
COPY operator/docker/ibm-qcs-example/bell.py /app/bell.py
ENV PYTHONUNBUFFERED=1
USER 65532:65532
ENTRYPOINT ["python", "/app/bell.py"]
