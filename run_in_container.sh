#!/bin/bash

# Run a provided command inside the manylinux image configured in pyproject.toml.

set -e

MANYLINUX_VERSION=$(make get-manylinux-version)

# Check if at least one parameter was provided
if [ $# -eq 0 ]; then
    echo "Usage: $0 <command> [args...]"
    echo 
    echo "Run a command inside the manylinux container."
    echo 
    echo "Examples:"
    echo "  $0 make libqrmi-tarball"
    echo "  $0 make build-wheels"
    echo 
    exit 1
fi

QRMI_DIR=/root/qrmi
CONTAINER_ENGINE=$(make get-container-engine)

# Always build a new manylinux container locally adding qrmi build dependency packages.
# The image is built only in the first time this script is called.
# After that, the overhead to run the provided command inside a container is negligible
# compared to running it in the host.

${CONTAINER_ENGINE} build \
    --volume $PWD:${QRMI_DIR}:z \
    --build-arg QRMI_DIR=${QRMI_DIR} \
    --build-arg MANYLINUX_VERSION=${MANYLINUX_VERSION} \
    -f docker/manylinux/Dockerfile \
    -t localhost/qrmi-${MANYLINUX_VERSION}:latest .

# Store container ID for cleanup
CONTAINER_ID=""
ATTACH_PID=""

# Trap SIGINT and forward to container
cleanup() {
    if [ -n "$CONTAINER_ID" ]; then        
        # If container is still running, send SIGTERM and delete the image
        if ${CONTAINER_ENGINE} inspect "$CONTAINER_ID" >/dev/null 2>&1; then
            echo "Stopping container..."
            ${CONTAINER_ENGINE} stop --time=5 "$CONTAINER_ID" 2>/dev/null
            echo "Deleting container..."
            ${CONTAINER_ENGINE} rm "$CONTAINER_ID" 2>/dev/null
        fi
    fi
    exit 130
}

trap cleanup SIGINT SIGTERM

# Run container in background and capture its ID
# The container is removed when it is finished.
CONTAINER_ID=$(${CONTAINER_ENGINE} run \
    -dit \
    --rm \
    --volume $PWD:${QRMI_DIR}:z \
    --env INSIDE_CONTAINER=1 \
    --workdir ${QRMI_DIR} \
    localhost/qrmi-${MANYLINUX_VERSION}:latest \
    bash -ic 'source ~/.bashrc && exec "$@"' -- "$@")

# Attach to container (this blocks until container exits or we get a signal)
${CONTAINER_ENGINE} attach "$CONTAINER_ID" &
ATTACH_PID=$!

# Wait for either the attach process or the container to finish
wait $ATTACH_PID 2>/dev/null
EXIT_CODE=$?

exit $EXIT_CODE
