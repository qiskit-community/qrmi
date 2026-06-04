# syntax=docker/dockerfile:1

# ---------------------------------------------------------------------------
# Build stage
# ---------------------------------------------------------------------------
FROM rust:1.91-slim AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev musl-tools \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Copy workspace manifests first to cache dependency downloads.
# rust-toolchain.toml must be present before any cargo/rustup invocation
# so the pinned toolchain is activated.
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY dependencies/ dependencies/
COPY operator/Cargo.toml operator/Cargo.toml

# Install the musl target here (rather than in rust-toolchain.toml) so
# only the container build pays the cost, not every contributor's rustup.
RUN rustup target add x86_64-unknown-linux-musl

# cbindgen.toml is required by the qrmi crate's build.rs
COPY cbindgen.toml .

# Fake source so cargo can fetch deps before copying real source.
RUN mkdir -p operator/src && echo 'fn main(){}' > operator/src/main.rs \
    && mkdir -p src && touch src/lib.rs

RUN cargo build --release --target x86_64-unknown-linux-musl -p qrmi-operator --locked 2>/dev/null || true

# Now copy real source and rebuild (only changed crates recompile).
COPY src/ src/
COPY build.rs .
COPY operator/src/ operator/src/

RUN cargo build --release --target x86_64-unknown-linux-musl -p qrmi-operator --locked

# ---------------------------------------------------------------------------
# Runtime stage — static musl binary needs no glibc; scratch is minimal
# ---------------------------------------------------------------------------
FROM gcr.io/distroless/static-debian12:nonroot

COPY --from=builder /build/target/x86_64-unknown-linux-musl/release/qrmi-operator /qrmi-operator

ENTRYPOINT ["/qrmi-operator"]
