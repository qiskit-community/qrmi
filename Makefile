# SPDX-License-Identifier: Apache-2.0
# (C) Copyright IBM Corporation 2026

QRMI_VERSION := $$($(MAKE) get-qrmi-version)
DIST_DIR ?= .

include Makefile_common.mk

.PHONY: all build dist dist-rhel-lib clean help

all: build

# ------------------------------------------------
# Rust targets
# ------------------------------------------------

build:
	cargo build --locked --release

# ------------------------------------------------
# Packaging targets
# ------------------------------------------------

# Allow disconnected/airgapped builds
tarball-vendor:
	cargo vendor $(DIST_DIR)/vendor
	@tar czf $(DIST_DIR)/qrmi-$(QRMI_VERSION)-vendor.tar.gz vendor/
	@rm -rf $(DIST_DIR)/vendor
	@echo
	@echo "Created: $(DIST_DIR)/qrmi-$(QRMI_VERSION)-vendor.tar.gz"

dist-rhel-lib: build
	TARBALL="$(DIST_DIR)/libqrmi-$(QRMI_VERSION)-el8-x86_64.tar.gz"; \
	WORKDIR="$(DIST_DIR)/libqrmi-$(QRMI_VERSION)"; \
	mkdir -p $$WORKDIR; \
	cp target/release/libqrmi.so $$WORKDIR; \
	cp qrmi.h $$WORKDIR; \
	cp LICENSE.txt $$WORKDIR; \
	tar czf $$TARBALL -C $(DIST_DIR) libqrmi-$(QRMI_VERSION); \
	rm -rf $$WORKDIR; \
	echo "Created: $$TARBALL"

# ------------------------------------------------
# Misc targets
# ------------------------------------------------

clean:
	cargo clean

help:
	@echo "Rust targets:"
	@echo
	@echo "  all    - same as \"make build\" (default)"
	@echo "  build  - Build libqrmi"
	@echo
	@echo "Packaging targets:"
	@echo
	@echo "  tarball-vendor  - Create vendor tarball in DIST_DIR (default: ./)"
	@echo "  dist-rhel-lib   - Create libqrmi tarball with libqrmi.so and qrmi.h"
	@echo
	@echo "Misc targets:"
	@echo
	@echo "  clean  - Remove build artifacts"
	@echo "  help   - Show this help message"
	