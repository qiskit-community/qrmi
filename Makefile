# SPDX-License-Identifier: Apache-2.0
# (C) Copyright IBM Corporation 2026

include Makefile.helpers

VERSION := $(shell grep -m1 '^version' Cargo.toml | sed 's/.*"\(.*\)".*/\1/')
DIST_DIR ?= /tmp

.PHONY: all build build-bins build-examples build-wheels install-wheels venv fmt fmt-wheels lint lint-bins lint-rust lint-wheels test-rust test-libqrmi test-bins test-dependencies test-wheels deps-rpm deps-deb dist rpm deb clean help

all: build

# ------------------------------------------------
# Rust targets
# ------------------------------------------------

build:
	cargo build --locked --release

build-bins:
	cargo build --bin task_runner --release --features="build-binary"
	cargo build --bin stubgen --release --features="pyo3"

build-examples:
	cargo build --examples --locked --release

# Format check libqrmi, dependencies, examples and binaries
fmt: check-rustfmt
	cargo fmt --all -- --check --verbose

lint: check-clippy
	cargo clippy --all-targets -- -D warnings

lint-bins: check-clippy
	cargo clippy --bin task_runner --features="build-binary" -- -D warnings
	cargo clippy --bin stubgen --features="pyo3" -- -D warnings

lint-rust: lint lint-bins

test-libqrmi:
	cargo test --locked --all-targets --release

test-bins:
	cargo test --locked --bin task_runner --release --features="build-binary"
	cargo test --locked --bin stubgen --release --features="pyo3"

test-dependencies:
	cargo test --locked --release -p direct-access-api
	cargo test --locked --release -p pasqal-cloud-api
	cargo test --locked --release -p qiskit_runtime_client

test-rust: test-libqrmi test-bins test-dependencies

# ------------------------------------------------
# Python targets
# ------------------------------------------------

venv:
	python3 -m venv .venv
	.venv/bin/pip install --upgrade pip
	.venv/bin/pip install -r requirements-dev.txt
	@echo "Virtual environment created. You can activate it with: source .venv/bin/activate"

build-wheels:
	maturin build --release

fmt-wheels: check-venv
	.venv/bin/black --check ./python

install-wheels: check-venv build-wheels
	.venv/bin/pip install --force-reinstall target/wheels/qrmi-*.whl

lint-wheels: install-wheels
	.venv/bin/pylint ./python

test-wheels: install-wheels
	.venv/bin/pytest python/tests/

# ------------------------------------------------
# Packaging targets
# ------------------------------------------------

dist:
	git archive --format=tar.gz \
	  --prefix=qrmi-$(VERSION)/ \
	  HEAD \
	  -o $(DIST_DIR)/qrmi-$(VERSION).tar.gz
	cargo vendor vendor
	tar czf $(DIST_DIR)/qrmi-$(VERSION)-vendor.tar.gz vendor/

rpm: dist
	mkdir -p ~/rpmbuild/{BUILD,BUILDROOT,RPMS,SOURCES,SPECS,SRPMS}
	cp $(DIST_DIR)/qrmi-$(VERSION).tar.gz        ~/rpmbuild/SOURCES/
	cp $(DIST_DIR)/qrmi-$(VERSION)-vendor.tar.gz ~/rpmbuild/SOURCES/
	cp packaging/rpm/qrmi.spec                    ~/rpmbuild/SPECS/
	rpmbuild -ba \
	  --define "version $(VERSION)" \
	  ~/rpmbuild/SPECS/qrmi.spec

deb: dist
	ln -sfn packaging/debian debian
	dch --newversion $(VERSION)-1 --distribution unstable \
	  --force-distribution "Release $(VERSION)"
	dpkg-buildpackage -us -uc -b
	rm -f debian

# ------------------------------------------------
# Setup targets
# ------------------------------------------------

deps-rpm: check-root check-rpm-distro
	dnf builddep -y packaging/rpm/qrmi.spec

deps-deb: check-root check-deb-distro
	mk-build-deps --install --remove packaging/debian/control

# ------------------------------------------------
# Misc targets
# ------------------------------------------------

clean:
	cargo clean

help:
	@echo "Rust targets:"
	@echo
	@echo "  all    - Build the project (default)"
	@echo "  build  - Build libqrmi"
	@echo "    build-bins      - Build task_runner and stubgen binaries"
	@echo "    build-examples  - Build Rust examples"
	@echo "  test-rust         - Run all Rust unit tests"
	@echo "    test-libqrmi    - Run Rust unit tests for libqrmi"
	@echo "    test-bins       - Run Rust unit tests for task_runner and stubgen binaries"
	@echo "    test-dependencies  - Run Rust unit tests for direct_access_client, pasqal_cloud_client and qiskit_runtime_client"
	@echo "  lint-rust          - Run all Rust lints (lint + lint-bins)"
	@echo "    lint             - Lint libqrmi, dependencies and examples"
	@echo "    lint-bins        - Lint task_runner and stubgen binaries"
	@echo "  fmt                - Check Rust code formatting for all targets"
	@echo
	@echo "Python targets:"
	@echo
	@echo "  build-wheels    - Build Python wheels using maturin"
	@echo "  install-wheels  - Install wheels into .venv"
	@echo "  venv            - Create .venv and install requirements-dev.txt"
	@echo "  fmt-wheels      - Check Python code formatting using black"
	@echo "  lint-wheels     - Lint Python code using pylint"
	@echo "  test-wheels     - Run Python unit tests"
	@echo
	@echo "Packaging targets:"
	@echo
	@echo "  dist   - Create source and vendor tarballs in DIST_DIR (default: /tmp)"
	@echo "  rpm    - Build RPM packages (requires dist tarballs in DIST_DIR)"
	@echo "  deb    - Build Debian packages (.deb)"
	@echo
	@echo "Setup targets:"
	@echo
	@echo "  deps-rpm   - Install RPM build dependencies from packaging/rpm/qrmi.spec"
	@echo "  deps-deb   - Install Debian build dependencies from packaging/debian/control"
	@echo
	@echo "Misc targets:"
	@echo
	@echo "  clean  - Remove build artifacts"
	@echo "  help   - Show this help message"
	