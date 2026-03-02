# SPDX-License-Identifier: Apache-2.0
# (C) Copyright IBM Corporation 2026

VERSION := $(shell grep -m1 '^version' Cargo.toml | sed 's/.*"\(.*\)".*/\1/')
DIST_DIR ?= .

include Makefile.helpers

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

dist:
	git archive --format=tar.gz \
	  --prefix=qrmi-$(VERSION)/ \
	  HEAD \
	  -o $(DIST_DIR)/qrmi-$(VERSION).tar.gz
	cargo vendor $(DIST_DIR)/vendor
	tar czf $(DIST_DIR)/qrmi-$(VERSION)-vendor.tar.gz vendor/
	rm -rf $(DIST_DIR)/vendor

dist-rhel-lib: build
	TARBALL="$(DIST_DIR)/libqrmi-$(VERSION)-el8-x86_64.tar.gz"; \
	WORKDIR="$(DIST_DIR)/libqrmi-$(VERSION)"; \
	mkdir -p $$WORKDIR; \
	cp target/release/libqrmi.so $$WORKDIR; \
	cp qrmi.h $$WORKDIR; \
	cp LICENSE.txt $$WORKDIR; \
	tar czf $$TARBALL -C $(DIST_DIR) libqrmi-$(VERSION); \
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
	@echo "  dist            - Create source and vendor tarballs in DIST_DIR (default: ./)"
	@echo "  dist-rhel-lib   - Create libqrmi tarball with libqrmi.so and qrmi.h"
	@echo
	@echo "Misc targets:"
	@echo
	@echo "  clean  - Remove build artifacts"
	@echo "  help   - Show this help message"
	