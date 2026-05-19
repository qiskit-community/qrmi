# SPDX-License-Identifier: Apache-2.0
# (C) Copyright IBM Corporation 2026

ifndef MAKEFILE_COMMON_MK_INCLUDED

.PHONY: get-qrmi-version check-new-qrmi-version-valid check-python-version-installed check-venv-exists check-python-devel-installed

ARCH := $(shell uname -m)
# Get QRMI version from Cargo.toml
QRMI_VERSION := $(shell grep -m1 '^version' Cargo.toml | sed 's/.*"\(.*\)".*/\1/')

MANYLINUX_VERSION ?= $(shell grep "manylinux-$(ARCH)-image" pyproject.toml | sed -E 's/.*=[[:space:]]*"(.*)"/\1/')

# Get python version from pyproject.toml
PYTHON_VERSION ?= $(shell grep "requires-python" pyproject.toml | cut -d'"' -f2 | sed 's/[^0-9.]//g')
PYTHON_VERSION_NO_DOTS = $(shell echo "$(PYTHON_VERSION)" | sed 's/\.//')
PYTHON_VENV_DIR = .venv_py$(PYTHON_VERSION_NO_DOTS)
PYTHON_VENV_ACTIVATE = $(PYTHON_VENV_DIR)/bin/activate

WHEELS_PATH = wheelhouse/qrmi-$(QRMI_VERSION)-cp$(PYTHON_VERSION_NO_DOTS)-abi3-$(MANYLINUX_VERSION)_$(ARCH).whl

LIBQRMI_SO_PATH = target/release/libqrmi.so
QRMI_H_PATH = qrmi.h

ifeq ($(INSIDE_CONTAINER),1)
CONTAINER_ENGINE = inside_container
else
CONTAINER_ENGINE = $(shell \
if command -v podman &> /dev/null; then echo "podman"; \
elif command -v docker &> /dev/null; then echo "docker"; \
else echo "none"; fi)
ifeq ($(CONTAINER_ENGINE),none)
$(error "No container engine found. Please install docker or podman.")
endif
endif

MAKEFLAGS += --no-print-directory

get-container-engine:
	@echo "$(CONTAINER_ENGINE)"

get-qrmi-version:
	@echo "$(QRMI_VERSION)"

get-venv-activate: $(PYTHON_VENV_DIR)
	@echo "$(PYTHON_VENV_ACTIVATE)"

get-python-version:
	@echo "$(PYTHON_VERSION)"

get-manylinux-version:
	@echo "$(MANYLINUX_VERSION)"

# Before releasing a new qrmi version, we check if the current version
# was not released yet in github and pypi.
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
	if curl -s https://pypi.org/pypi/qrmi/json | jq -r '.releases | keys[]' | grep -q "$${NEW_QRMI_VERSION}"; then \
		echo "Error: Release v$${NEW_QRMI_VERSION} already exists in PyPI"; \
		echo "Please update the version in Cargo.toml and try again"; \
		exit 1; \
	fi; \
	echo "New QRMI version v$${NEW_QRMI_VERSION} is valid"

check-python-version-installed:
	@if ! command -v python$(PYTHON_VERSION) >/dev/null 2>&1 ; then \
		echo "Error: python$(PYTHON_VERSION) not found"; \
		echo "Please install the package python$(PYTHON_VERSION)"; \
		exit 1; \
	fi

check-python-devel-installed:
	@if command -v dpkg >/dev/null 2>&1 ; then \
	    if ! dpkg -l python$(PYTHON_VERSION)-dev >/dev/null 2>&1 ; then \
			echo "Error: package python$(PYTHON_VERSION)-dev not found, please install it"; \
			exit 1; \
		fi \
	elif command -v rpm >/dev/null 2>&1 ; then \
	    if ! rpm -q python$(PYTHON_VERSION)-devel >/dev/null 2>&1 ; then \
			echo "Error: package python$(PYTHON_VERSION)-devel not found, please install it"; \
			exit 1; \
		fi \
	fi

check-doxygen-installed:
	@if ! command -v doxygen >/dev/null 2>&1 ; then \
		echo "Error: doxygen not found"; \
		echo "Please install the package doxygen"; \
		exit 1; \
	fi

MAKEFILE_COMMON_MK_INCLUDED := true
endif