# SPDX-License-Identifier: Apache-2.0
# (C) Copyright IBM Corporation 2026

ifndef MAKEFILE_COMMON_MK_INCLUDED

.PHONY: get-qrmi-version check-new-qrmi-version-valid

# Get QRMI version from Cargo.toml
QRMI_VERSION := $(shell grep -m1 '^version' Cargo.toml | sed 's/.*"\(.*\)".*/\1/')

MAKEFLAGS += --no-print-directory

# Single source of truth to get the qrmi version.
get-qrmi-version:
	@echo "$(QRMI_VERSION)"

# Before releasing a new qrmi version, we check if the current version was not released yet
check-new-qrmi-version-valid:
	@NEW_QRMI_VERSION=$$($(MAKE) get-qrmi-version); \
	if [ -z "$${NEW_QRMI_VERSION}" ]; then \
		echo "Error: Failed to get QRMI version from Cargo.toml"; \
		exit 1; \
	fi; \
	HTTP_CODE=$$(curl -s -o /dev/null -w "%{http_code}" -H "Accept: application/vnd.github+json" "https://api.github.com/repos/qiskit-community/qrmi/releases/tags/v$${NEW_QRMI_VERSION}"); \
	if [ "$${HTTP_CODE}" = "200" ]; then \
		echo "Error: Release v$${NEW_QRMI_VERSION} already exists in GitHub releases"; \
		echo "Please update the version in Cargo.toml and try again"; \
		exit 1; \
	elif [ "$${HTTP_CODE}" != "404" ]; then \
		echo "Error: Failed to check GitHub releases (HTTP $${HTTP_CODE})"; \
		exit 1; \
	fi; \
	echo "New QRMI version v$${NEW_QRMI_VERSION} is valid"

MAKEFILE_COMMON_MK_INCLUDED := true
endif