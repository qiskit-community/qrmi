# SPDX-License-Identifier: Apache-2.0
# (C) Copyright IBM Corporation 2026

.PHONY: check_root get-qrmi-version

MAKEFLAGS += --no-print-directory

# Single source of truth to get the qrmi version.
get-qrmi-version:
	@grep -m1 '^version' Cargo.toml | sed 's/.*"\(.*\)".*/\1/'

check-root:
	@if [ "$(shell id -u)" -ne 0 ]; then \
	  echo "run $(shell id -u)"; \
	  echo "Error: this target must be run as root (e.g. sudo make $(MAKECMDGOALS))"; \
	  exit 1; \
	fi