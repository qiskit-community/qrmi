#!/usr/bin/env bash
#
# Regenerate the Felis API client (src/apis, src/models, docs) with openapi-generator.
#
# Usage: ./regenerate.sh <path-or-url-to-felis-openapi-spec>
#
# Environment:
#   OPENAPI_GENERATOR_VERSION  generator image tag (default: v7.18.0)
#   CONTAINER_ENGINE           docker or podman (default: docker, else podman)
#
# Files listed in .openapi-generator-ignore (Cargo.toml, README.md, src/lib.rs)
# and hand-written files the generator doesn't produce (src/helpers.rs,
# src/jobs.rs, tests/) are left untouched.

set -euo pipefail

if [ $# -ne 1 ]; then
    echo "Usage: $0 <path-or-url-to-felis-openapi-spec>" >&2
    exit 1
fi

SPEC="$1"
VERSION="${OPENAPI_GENERATOR_VERSION:-v7.18.0}"
ENGINE="${CONTAINER_ENGINE:-$(command -v docker >/dev/null && echo docker || echo podman)}"
CRATE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# A local spec file must be mounted into the container.
SPEC_MOUNT=()
SPEC_ARG="$SPEC"
if [ -f "$SPEC" ]; then
    SPEC_DIR="$(cd "$(dirname "$SPEC")" && pwd)"
    SPEC_MOUNT=(-v "$SPEC_DIR:/spec:ro,Z")
    SPEC_ARG="/spec/$(basename "$SPEC")"
fi

# Remove previously generated files so that endpoints/models dropped from the
# spec don't linger.
rm -rf "$CRATE_DIR/src/apis" "$CRATE_DIR/src/models" "$CRATE_DIR/docs"

"$ENGINE" run --rm \
    --user "$(id -u):$(id -g)" \
    -v "$CRATE_DIR:/local:Z" \
    "${SPEC_MOUNT[@]}" \
    "docker.io/openapitools/openapi-generator-cli:$VERSION" generate \
    -g rust \
    -i "$SPEC_ARG" \
    -o /local \
    --package-name qrmi-alice-bob-felis-client \
    --additional-properties=supportMiddleware=false,library=reqwest

cargo fmt --manifest-path "$CRATE_DIR/Cargo.toml"

echo "Done. Review the diff, then run: cargo test -p qrmi-alice-bob-felis-client"
