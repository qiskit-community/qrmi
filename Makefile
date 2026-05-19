# SPDX-License-Identifier: Apache-2.0
# (C) Copyright IBM Corporation 2026

DIST_DIR ?= .
SHELL := /bin/bash

default: build

include Makefile_common.mk

# ------------------------------------------------
# Build targets
# ------------------------------------------------

.PHONY: build build-rust-examples build-task-runner build-stubgen
.PHONY: build-c-examples build-wheels build-rust-all

LIBQRMI_SO_PATH : build

build:
	cargo build --locked --release --lib

build-rust-examples:
	cargo build --locked --release --examples

build-task-runner:
	cargo build --locked --release --bin task_runner --features="build-binary"

build-stubgen: check-python-devel-installed
ifeq ($(INSIDE_CONTAINER),1)
	$(error "Manylinux images don't come with libpython, please run the command without ./run_in_container.sh")
endif
	PYO3_PYTHON=python$(PYTHON_VERSION) cargo build --locked --release --bin stubgen --features="pyo3"

build-c-examples: $(LIBQRMI_SO_PATH)
	@mkdir -p examples/qrmi/c/ibm_quantum_system/build
	@cd examples/qrmi/c/ibm_quantum_system/build && \
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

$(WHEELS_PATH):
	@source $(PYTHON_VENV_ACTIVATE) && \
	CIBW_CONTAINER_ENGINE=$(CONTAINER_ENGINE) CIBW_TEST_SKIP='cp*' cibuildwheel

build-wheels: $(PYTHON_VENV_DIR) $(WHEELS_PATH)

# build-stubgen has very specific requirements, please do not include it in this target
build-rust-all: build build-rust-examples build-task-runner

# ------------------------------------------------
# Linting targets
# ------------------------------------------------

.PHONY: lint lint-rust-examples lint-task-runner lint-stubgen
.PHONY: lint-wheels lint-rust-all

lint:
	cargo clippy --locked --release --lib -- -D warnings

lint-rust-examples:
	cargo clippy --locked --release --examples -- -D warnings

lint-task-runner:
	cargo clippy --locked --release --bin task_runner --features="build-binary" -- -D warnings

lint-stubgen: check-python-devel-installed
ifeq ($(INSIDE_CONTAINER),1)
	$(error "Manylinux images don't come with libpython, please run the command without ./run_in_container.sh")
endif
	PYO3_PYTHON=python$(PYTHON_VERSION) cargo clippy --locked --release --bin stubgen --features="pyo3" -- -D warnings

lint-wheels: $(PYTHON_VENV_DIR) install-wheels
	@source $(PYTHON_VENV_ACTIVATE) && \
	pylint ./python

# lint-stubgen has very specific requirements, please do not include it in this target
lint-rust-all: lint lint-rust-examples lint-task-runner

# ------------------------------------------------
# Unit test targets
# ------------------------------------------------

.PHONY: test test-doc test-deps test-rust-examples test-task-runner
.PHONY: test-stubgen test-wheels test-rust-all

test:
	cargo test --lib --locked --release

test-doc:
	cargo test --doc --locked --release

test-deps:
	cargo test --locked --release -p quantum-system-api
	cargo test --locked --release -p pasqal-cloud-api
	cargo test --locked --release -p qiskit_runtime_client

test-rust-examples:
	cargo test --examples --locked --release

test-task-runner:
	cargo test --locked --release --bin task_runner --features="build-binary"

test-stubgen: check-python-devel-installed
ifeq ($(INSIDE_CONTAINER),1)
	$(error "Manylinux images don't come with libpython, please run the command without ./run_in_container.sh")
endif
	PYO3_PYTHON=python$(PYTHON_VERSION) cargo test --locked --release --bin stubgen --features="pyo3"

test-wheels: $(PYTHON_VENV_DIR)
	@source $(PYTHON_VENV_ACTIVATE) && \
	CIBW_CONTAINER_ENGINE=$(CONTAINER_ENGINE) CIBW_TEST_EXTRAS=all cibuildwheel

# test-stubgen has very specific requirements, please do not include it in this target
test-rust-all: test test-doc test-deps test-rust-examples test-task-runner

# ------------------------------------------------
# Format check targets
# ------------------------------------------------

.PHONY: fmt fmt-rust fmt-python

fmt: fmt-rust

fmt-rust:
	cargo fmt --all -- --check --verbose

fmt-python: $(PYTHON_VENV_DIR)
	@source $(PYTHON_VENV_ACTIVATE) && \
	black --check --target-version py$(PYTHON_VERSION_NO_DOTS) ./python

# ------------------------------------------------
# Setup targets
# ------------------------------------------------

.PHONY: create-venv install-wheels

$(PYTHON_VENV_DIR):
	@python$(PYTHON_VERSION) -m venv $(PYTHON_VENV_DIR) && \
	source $(PYTHON_VENV_ACTIVATE) && \
	pip install --upgrade pip && \
	pip install -r requirements-dev.txt && \
	echo && \
	echo "*** Virtual environment created ***" && \
	echo && \
	echo "The makefile targets will activate the venv automatically, but if you want" && \
	echo "you can manually activate it with: source $(PYTHON_VENV_DIR)/bin/activate" && \
	echo

create-venv: $(PYTHON_VENV_DIR) check-python-version-installed

# ------------------------------------------------

install-wheels: $(PYTHON_VENV_DIR) $(WHEELS_PATH)
	@source $(PYTHON_VENV_ACTIVATE) && \
	pip install --force-reinstall $(WHEELS_PATH)[all]

# ------------------------------------------------
# Clean targets
# ------------------------------------------------

.PHONY: clean clean-c-examples clean-tarballs clean-wheels clean-all

clean:
	cargo clean
	rm -f qrmi.h

clean-c-examples:
	rm -rf examples/qrmi/c/ibm_quantum_system/build
	rm -rf examples/qrmi/c/qiskit_runtime_service/build
	rm -rf examples/qrmi/c/pasqal_cloud/build
	rm -rf examples/qrmi/c/config/build

clean-tarballs:
	rm -f $(DIST_DIR)/libqrmi-$(QRMI_VERSION)-el8-x86_64.tar.gz
	rm -f $(DIST_DIR)/qrmi-$(QRMI_VERSION)-vendor.tar.gz

clean-wheels:
	rm -rf $(DIST_DIR)/wheelhouse

clean-docs:
	rm -rf $(DIST_DIR)/html

clean-all: clean clean-c-examples clean-tarballs clean-wheels

# ------------------------------------------------
# Documentation targets
# ------------------------------------------------

.PHONY: doc doc-python doc-c

doc:
	cargo doc --no-deps --open

doc-python: $(PYTHON_VENV_DIR)
	@source $(PYTHON_VENV_ACTIVATE) && \
	python -m pydoc -b

doc-c: check-doxygen-installed
	@doxygen Doxyfile
	@echo
	@echo "Open the file $(DIST_DIR)/html/index.html"

# ------------------------------------------------
# Packaging targets
# ------------------------------------------------

.PHONY: vendor-tarball libqrmi-tarball pypi-sdist

vendor-tarball:
	cargo vendor $(DIST_DIR)/vendor
	@tar czf $(DIST_DIR)/qrmi-$(QRMI_VERSION)-vendor.tar.gz vendor/
	@rm -rf $(DIST_DIR)/vendor
	@echo
	@echo "Created: $(DIST_DIR)/qrmi-$(QRMI_VERSION)-vendor.tar.gz"

libqrmi-tarball: LIBQRMI_SO_PATH
	@TARBALL="$(DIST_DIR)/libqrmi-$(QRMI_VERSION)-el8-$(ARCH).tar.gz" && \
	WORKDIR="$(DIST_DIR)/libqrmi-$(QRMI_VERSION)" && \
	mkdir -p $$WORKDIR && \
	cp $(LIBQRMI_SO_PATH) $$WORKDIR && \
	cp $(QRMI_H_PATH) $$WORKDIR && \
	cp LICENSE.txt $$WORKDIR && \
	tar czf $$TARBALL -C $(DIST_DIR) libqrmi-$(QRMI_VERSION) && \
	rm -rf $$WORKDIR

pypi-sdist: $(PYTHON_VENV_DIR)
	@source $(PYTHON_VENV_ACTIVATE) && \
	maturin sdist

# ------------------------------------------------
# Other targets
# ------------------------------------------------

.PHONY: help

define HELP_TEXT
Usage: make <target>

As shown below, some targets can be executed within a ${MANYLINUX_VERSION} container using the "./run_in_container.sh" script.

Main reasons why you may want to use "./run_in_container.sh":
1. Negligible time overhead. In the first call, some time will be spent to fetch the manylinux image and rebuild it
   locally adding some tooling for QRMI, however, after that running a make command with or without
   "./run_in_container.sh" is almost the same.
2. Artifacts are produced the same way as in the CI - release, on-pull-request, on-schedule, etc.
3. Less packages to install and configure in your system.

Targets that leverages cibuildwheel cannot be executed using "./run_in_container.sh". No need to have nested containers ;-)

Build targets:
    [./run_in_container.sh] build                - Build libqrmi (default target)
    [./run_in_container.sh] build-rust-examples  - Build rust examples
    [./run_in_container.sh] build-c-examples     - Build C examples
    [./run_in_container.sh] build-task-runner    - Build task_runner binary
                            build-stubgen        - Build stubgen binary
                            build-wheels         - Build qrmi wheels using cibuildwheel
    [./run_in_container.sh] build-rust-all       - Build libqrmi, rust examples and binaries (except stubgen)

Linting targets:
    [./run_in_container.sh] lint                 - Lint libqrmi
    [./run_in_container.sh] lint-rust-examples   - Lint rust examples
    [./run_in_container.sh] lint-task-runner     - Lint task_runner binary
                            lint-stubgen         - Lint stubgen binary
    [./run_in_container.sh] lint-wheels          - Lint built wheels with python-$(PYTHON_VERSION) (cibuildwheel not required)
                                                   The qrmi python package will be built and installed if necessary
    [./run_in_container.sh] lint-rust-all        - Lint libqrmi, rust examples and binaries (except stubgen)

Unit test targets:
    [./run_in_container.sh] test                 - Test libqrmi
    [./run_in_container.sh] test-deps            - Test libqrmi dependencies
    [./run_in_container.sh] test-doc             - Test libqrmi doctests
    [./run_in_container.sh] test-rust-examples   - Test rust examples
    [./run_in_container.sh] test-task-runner     - Test task_runner binary
                            test-stubgen         - Test stubgen binary
                            test-wheels          - Build and test wheels with all python >=$(PYTHON_VERSION) using cibuildwheel
                                                   The qrmi python package will be built and installed if necessary
    [./run_in_container.sh] test-rust-all        - Test libqrmi, doctests, rust examples and binaries (except stubgen)

Format check targets:
    [./run_in_container.sh] fmt                  - Same as "fmt-rust"
    [./run_in_container.sh] fmt-rust             - Format check libqrmi, dependencies, examples and binaries
    [./run_in_container.sh] fmt-python           - Format check the ./python directory with python-$(PYTHON_VERSION)

Clean targets:
    [./run_in_container.sh] clean                - Remove ./target directory and qrmi.h
    [./run_in_container.sh] clean-c-examples     - Remove C examples build directories
    [./run_in_container.sh] clean-tarballs       - Remove tarballs generated
    [./run_in_container.sh] clean-all            - Remove all artifacts built

Documentation targets:
    [./run_in_container.sh] doc                  - Generate rust API documentation and open it in a browser
    [./run_in_container.sh] doc-python           - Generate python API documentation and open it in a browser
    [./run_in_container.sh] doc-c                - Generate C API documentation

Setup targets:
    [./run_in_container.sh] create-venv          - Create python virtual environment for QRMI using the python version
                                                   defined in PYTHON_VERSION (default: 'make get-python-version').
                                                   Once created, the makefile target will activate the venv automatically.
    [./run_in_container.sh] install-wheels       - Install the wheels in the venv

Packaging targets:
    [./run_in_container.sh] libqrmi-tarball      - Create versioned libqrmi tarball with libqrmi.so and qrmi.h
                                                   for RHEL8 compatible distributions
    [./run_in_container.sh] vendor-tarball       - Create versioned vendor tarball in DIST_DIR (default: ./).
                                                   It can be used to build the qrmi version locally without
                                                   relying on the vendor crates to be available upstream.
                            pypi-sdist           - Create source distribution tarball for PyPI    

Other targets:
    [./run_in_container.sh] check-new-qrmi-version-valid   - Check if the qrmi version returned by "get-qrmi-version"
                                                             has already been released in github.
    [./run_in_container.sh] check-python-version-installed - Check if the RHEL 8 required python packages are installed
                            get-qrmi-version      - Read the qrmi version from Cargo.toml and print it
                            get-python-version    - Read the python version from pyproject.toml and print it
                            get-manylinux-version - Read the manylinux version from pyproject.toml and print it
                            get-venv-activate     - Print the path for the venv activate binary. Tip: 'source $$(make get-venv-activate)'
    [./run_in_container.sh] help                  - Show this help message


Examples:

Build libqrmi.so and qrmi.h inside a ${MANYLINUX_VERSION} container.
$$ ./run_in_container.sh make build

Build wheels using cibuildwheel as defined in pyproject.toml (python ${PYTHON_VERSION} and ${MANYLINUX_VERSION}).
The wheel is saved in ./wheelhouse/
$$ make build-wheels

Save the latest QRMI version in a bash variable.
$$ QRMI_VERSION=$$(make get-qrmi-version)

Activate the venv created by the makefile. The default python is ${PYTHON_VERSION} as defined in the pyproject.toml.
$$ source $$(make get-venv-activate)

Create a venv for a custom python version.
$$ PYTHON_VERSION=3.13 make create-venv

Activate a customized venv created by the makefile.
$$ source $$(PYTHON=3.12 make get-venv-activate)

Create the libqrmi tarball. It will be saved in $(DIST_DIR)/libqrmi-$(QRMI_VERSION)-el8-$(ARCH).tar.gz
$$ ./run_in_container.sh make libqrmi-tarball

endef
export HELP_TEXT

help:
	@echo "$$HELP_TEXT"
