# Add qrmi build dependency packages to the provided manylinux image.
#
# We use the image built locally to run make commands with negligible time overhead, see "make help".

ARG MANYLINUX_VERSION=manylinux_2_28

FROM quay.io/pypa/${MANYLINUX_VERSION}

ARG QRMI_DIR=/root/qrmi

ENV INSIDE_CONTAINER=1

WORKDIR ${QRMI_DIR}

RUN \
  curl -sSf https://sh.rustup.rs | sh -s -- -y && \
  source ~/.cargo/env && \
  make build && \
  make clean
