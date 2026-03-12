# SPDX-License-Identifier: Apache-2.0
# (C) Copyright IBM Corporation 2026

.PHONY: get-qrmi-version

MAKEFLAGS += --no-print-directory

# Single source of truth to get the qrmi version.
get-qrmi-version:
	@grep -m1 '^version' Cargo.toml | sed 's/.*"\(.*\)".*/\1/'

