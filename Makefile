# SPDX-License-Identifier: Apache-2.0
# (C) Copyright IBM Corporation 2026

DIST_DIR ?= .

.PHONY: _build
_build: build

include Makefile_common.mk

# ------------------------------------------------
# Build targets
# ------------------------------------------------

.PHONY: build build-rust-examples build-task-runner build-stubgen
.PHONY: build-c-examples build-pypkg build-rust-all build-all

build:
	cargo build --locked --release --lib

build-rust-examples:
	cargo build --locked --release --examples

build-task-runner:
	cargo build --locked --release --bin task_runner --features="build-binary"

build-stubgen: check-venv-exists
	@source $(PYTHON_VENV_ACTIVATE) && \
	cargo build --locked --release --bin stubgen --features="pyo3"

build-c-examples: build
	@mkdir -p examples/qrmi/c/direct_access/build
	@cd examples/qrmi/c/direct_access/build && \
		cmake -DCMAKE_BUILD_TYPE=Release .. && \
		cmake --build .
	@mkdir -p examples/qrmi/c/qiskit_runtime_service/build
	@cd examples/qrmi/c/qiskit_runtime_service/build && \
		cmake -DCMAKE_BUILD_TYPE=Release .. && \
		cmake --build .
	@mkdir -p examples/qrmi/c/pasqal_cloud/build
	@cd examples/qrmi/c/pasqal_cloud/build && \
		cmake -DCMAKE_BUILD_TYPE=Release .. && \
		cmake --build .
	@mkdir -p examples/qrmi/c/config/build
	@cd examples/qrmi/c/config/build && \
		cmake -DCMAKE_BUILD_TYPE=Release .. && \
		cmake --build .

build-pypkg: check-venv-exists
	@source $(PYTHON_VENV_ACTIVATE) && \
	maturin build --locked --release

build-rust-all: build build-rust-examples build-task-runner build-stubgen

build-all: build-rust-all build-c-examples build-pypkg

# ------------------------------------------------
# Linting targets
# ------------------------------------------------

.PHONY: lint lint-rust-examples lint-task-runner lint-stubgen
.PHONY: lint-pypkg lint-rust-all lint-all

lint:
	cargo clippy --locked --release --lib -- -D warnings

lint-rust-examples:
	cargo clippy --locked --release --examples -- -D warnings

lint-task-runner:
	cargo clippy --locked --release --bin task_runner --features="build-binary" -- -D warnings

lint-stubgen: check-venv-exists
	@source $(PYTHON_VENV_ACTIVATE) && \
	cargo clippy --locked --release --bin stubgen --features="pyo3" -- -D warnings

lint-pypkg: check-venv-exists install-pypkg
	@source $(PYTHON_VENV_ACTIVATE) && \
	pylint ./python

lint-rust-all: lint-with-examples lint-task-runner lint-stubgen

lint-all: lint-rust-all lint-pypkg

# ------------------------------------------------
# Unit test targets
# ------------------------------------------------

.PHONY: test test-doc test-deps test-rust-examples test-task-runner
.PHONY: test-stubgen test-pypkg test-rust-all test-all

test:
	cargo test --lib --locked --release

test-doc:
	cargo test --doc --locked --release

test-deps:
	cargo test --locked --release -p direct-access-api
	cargo test --locked --release -p pasqal-cloud-api
	cargo test --locked --release -p qiskit_runtime_client

test-rust-examples:
	cargo test --examples --locked --release

test-task-runner:
	cargo test --locked --release --bin task_runner --features="build-binary"

test-stubgen: check-venv-exists
	@source $(PYTHON_VENV_ACTIVATE) && \
	cargo test --locked --release --bin stubgen --features="pyo3"

test-pypkg: check-venv-exists install-pypkg
	@source $(PYTHON_VENV_ACTIVATE) && \
	pytest python/tests/

test-rust-all: test test-rust-examples test-rust-doc test-task-runner test-stubgen

test-all: test-rust-all test-pypkg

# ------------------------------------------------
# Format check targets
# ------------------------------------------------

.PHONY: fmt fmt-rust fmt-pypkg fmt-all

fmt: fmt-rust

fmt-rust:
	cargo fmt --all -- --check --verbose

fmt-pypkg: check-venv-exists
	@source $(PYTHON_VENV_ACTIVATE) && \
	black --check ./python

fmt-all: fmt-rust fmt-pypkg

# ------------------------------------------------
# Setup targets
# ------------------------------------------------

.PHONY: create-env install-pypkg

create-venv: check-python-version-installed
	@if [ -d $(PYTHON_VENV_DIR) ]; then \
	  echo "Error: $(PYTHON_VENV_DIR) already exists. Remove it first or just skip this step."; \
	  exit 1; \
	fi; \
	python$(PYTHON_VERSION) -m venv $(PYTHON_VENV_DIR) && \
	source $(PYTHON_VENV_ACTIVATE) && \
	pip install --upgrade pip && \
	pip install -r requirements-dev.txt && \
	echo && \
	echo "*** Virtual environment created ***" && \
	echo && \
	echo "The makefile targets will activate the venv automatically, but if you want" && \
	echo "you can manually activate it with: source $(PYTHON_VENV_DIR)/bin/activate" && \
	echo

install-pypkg: check-venv-exists
	@source $(PYTHON_VENV_ACTIVATE) && \
	maturin develop --locked --release

# ------------------------------------------------
# Clean targets
# ------------------------------------------------

.PHONY: clean clean-c-examples clean-tarballs

clean:
	cargo clean
	@rm -f qrmi.h

clean-c-examples:
	@rm -rf examples/qrmi/c/direct_access/build
	@rm -rf examples/qrmi/c/qiskit_runtime_service/build
	@rm -rf examples/qrmi/c/pasqal_cloud/build
	@rm -rf examples/qrmi/c/config/build

clean-tarballs:
	rm -f $(DIST_DIR)/libqrmi-$(QRMI_VERSION)-el8-x86_64.tar.gz
	rm -f $(DIST_DIR)/qrmi-$(QRMI_VERSION)-vendor.tar.gz

clean-docs:
	rm -rf $(DIST_DIR)/html

# ------------------------------------------------
# Documentation targets
# ------------------------------------------------

.PHONY: doc doc-pypkg doc-c

doc:
	cargo doc --no-deps --open

doc-pypkg: check-venv-exists
	@source $(PYTHON_VENV_ACTIVATE) && \
	python -m pydoc -b

doc-c: check-doxygen-installed
	@doxygen Doxyfile
	@echo
	@echo "Open the file $(DIST_DIR)/html/index.html"

# ------------------------------------------------
# Packaging targets
# ------------------------------------------------

.PHONY: tarball-vendor tarball-libqrmi-el8 clean-tarballs

tarball-vendor:
	cargo vendor $(DIST_DIR)/vendor
	@tar czf $(DIST_DIR)/qrmi-$(QRMI_VERSION)-vendor.tar.gz vendor/
	@rm -rf $(DIST_DIR)/vendor
	@echo
	@echo "Created: $(DIST_DIR)/qrmi-$(QRMI_VERSION)-vendor.tar.gz"

tarball-libqrmi-el8: build
	@TARBALL="$(DIST_DIR)/libqrmi-$(QRMI_VERSION)-el8-x86_64.tar.gz" && \
	WORKDIR="$(DIST_DIR)/libqrmi-$(QRMI_VERSION)" && \
	mkdir -p $$WORKDIR && \
	cp target/release/libqrmi.so $$WORKDIR && \
	cp qrmi.h $$WORKDIR && \
	cp LICENSE.txt $$WORKDIR && \
	tar czf $$TARBALL -C $(DIST_DIR) libqrmi-$(QRMI_VERSION) && \
	rm -rf $$WORKDIR && \
	echo "Created: $$TARBALL"

# ------------------------------------------------
# Other targets
# ------------------------------------------------

define HELP_TEXT
Usage: make <target>

Build targets:
    build                - Build libqrmi (default target)
    build-rust-examples  - Build rust examples
    build-c-examples     - Build C examples
    build-task-runner    - Build task_runner binary
    build-stubgen        - Build stubgen binary. Requires: create-venv
    build-pypkg          - Build qrmi python package. Requires: create-venv
    build-rust-all       - Build libqrmi, rust examples and binaries
    build-all            - Build everything!

Linting targets:
    lint                 - Lint libqrmi
    lint-rust-examples   - Lint rust examples
    lint-task-runner     - Lint task_runner binary
    lint-stubgen         - Lint stubgen binary. (Requires: create-venv)
    lint-pypkg           - Lint qrmi python package installed in the venv. (Requires: create-venv)
                           The qrmi python package will be built and installed if necessary
    lint-rust-all        - Lint libqrmi, rust examples and binaries
    lint-all             - Lint everything!

Unit test targets:
    test                 - Test libqrmi
    test-deps            - Test libqrmi dependencies
    test-doc             - Test libqrmi doctests
    test-rust-examples   - Test rust examples
    test-task-runner     - Test task_runner binary
    test-stubgen         - Test stubgen binary. (Requires: create-venv)
    test-pypkg           - Test the qrmi python package installed in the venv. (Requires: create-venv)
                           The qrmi python package will be built and installed if necessary
    test-rust-all        - Test libqrmi, doctests, rust examples and binaries
    test-all             - Test everything!

Format check targets:
    fmt                  - Same as "fmt-rust"
    fmt-rust             - Format check libqrmi, dependencies, examples and binaries
    fmt-pypkg            - Format check the ./python directory. (Requires: create-venv)
    fmt-all              - Format check everything!

Clean targets:
    clean                - Remove ./target directory and qrmi.h
    clean-c-examples     - Remove C examples build directories
    clean-tarballs       - Remove tarballs generated
    clean-all            - Remove all artifacts built

Documentation targets:
    doc                  - Generate rust API documentation and open it in a browser
    doc-pypkg            - Generate python API documentation and open it in a browser. (Requires: create-venv)
    doc-c                - Generate C API documentation

Setup targets:
    create-venv          - Create python virtual environment for QRMI using the python version
                           defined in PYTHON_VERSION (default: "3.12").
                           Once created, the makefile target will activate the venv automatically.
    install-pypkg        - Build (if needed) and install the qrmi python package

Packaging targets:
    tarball-libqrmi-el8  - Create versioned libqrmi tarball with libqrmi.so and qrmi.h
                           for RHEL8 compatible distributions
    tarball-vendor       - Create versioned vendor tarball in DIST_DIR (default: ./).
                           It can be used to build the qrmi version locally without
                           relying on the vendor crates to be available upstream.

Other targets:
    check-new-qrmi-version-valid   - Check if the qrmi version returned by "get-qrmi-version"
                                     has already been released in github.
    check-python-version-installed - Check if the RHEL 8 required python packages are installed
    check-venv-exists    - Check if the venv for PYTHON_VERSION (default: "3.12") has already been created
    get-qrmi-version     - Read the qrmi version from Cargo.toml and print it
    help                 - Show this help message

endef
export HELP_TEXT

.PHONY: clean-all help

clean-all: clean clean-c-examples clean-tarballs

help:
	@echo "$$HELP_TEXT"
