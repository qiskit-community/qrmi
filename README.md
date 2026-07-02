# Quantum Resource Management Interface (QRMI)

[![License](https://img.shields.io/github/license/qiskit-community/qrmi.svg?)](https://opensource.org/licenses/Apache-2.0) <!--- long-description-skip-begin -->
[![Current Release](https://img.shields.io/github/release/qiskit-community/qrmi.svg?)](https://github.com/qiskit-community/qrmi/releases)
![Platform](https://img.shields.io/badge/%F0%9F%92%BB_Platform-Linux%20%7C%20macOS-blue)
![PyPI - Python Version](https://img.shields.io/pypi/pyversions/qrmi)
[![Minimum rustc 1.91](https://img.shields.io/badge/rustc-1.91+-blue.svg)](https://rust-lang.github.io/rfcs/2495-min-rust-version.html)
[![Downloads](https://img.shields.io/pypi/dm/qrmi.svg)](https://pypi.org/project/qrmi/)
[![Downloads](https://static.pepy.tech/badge/qrmi)](https://pepy.tech/project/qrmi)<!--- long-description-skip-end -->
[![DOI](https://zenodo.org/badge/DOI/10.5281/zenodo.20650771.svg)](https://doi.org/10.5281/zenodo.20650771)
[![arXiv](https://img.shields.io/badge/arXiv-2506.10052-b31b1b.svg)](https://arxiv.org/abs/2506.10052)
[![CI](https://github.com/qiskit-community/qrmi/actions/workflows/on-schedule.yml/badge.svg)](https://github.com/qiskit-community/qrmi/actions/workflows/on-schedule.yml)

The *Quantum Resource Management Interface* (QRMI) is a vendor-agnostic library for high-performance compute (HPC) systems to access, control, and monitor the behavior of quantum computational resources. It acts as a thin middleware layer that abstracts away the complexities associated with controlling quantum resources through a set of simple APIs. Written in Rust, this interface also exposes Python and C APIs for ease of integration into nearly any computational environment.

The source code to build and deploy QRMI is available [here](https://github.com/qiskit-community/qrmi).

An optional `task_runner` command line tool to execute quantum payloads against quantum hardware is included in the Python package. For more information, read the documentation available [here](https://github.com/qiskit-community/qrmi/blob/main/python/qrmi/tools/task_runner/README.md).

## Features

TBD

## Installation

### Python

We encourage installing QRMI via ``pip``:

```bash
pip install qrmi
```

To use a specific quantum resource, install QRMI with the corresponding optional dependencies:

```bash
pip install "qrmi[ibm]"       # Include dependencies for IBM
pip install "qrmi[iqm]"       # Include dependencies for IQM
pip install "qrmi[pasqal]"    # Include dependencies for Pasqal
pip install "qrmi[alice-bob]" # Include dependencies for Alice and Bob
pip install "qrmi[all]"       # Include dependencies for all quantum resources except `alice-bob`
```

Or combine multiple resources:

```bash
pip install "qrmi[ibm,pasqal]"
```

> [!NOTE]
> `ibm` and `iqm` extras cannot be installed together, as they depend on incompatible versions of Qiskit.

> [!NOTE]
> `alice-bob` cannot be installed alongside `ibm` or `iqm`, as it depends on Qiskit versions earlier than 2.0.

Pip will handle all dependencies automatically and you will always install the latest (and most thoroughly tested) version.

To install from source, follow the instructions in the [documentation](https://github.com/qiskit-community/qrmi/blob/main/INSTALL.md).

### Standalone C library

Prebuilt binaries for  (glibc 2.28 compatible) on x86_64, ppc64le, and aarch64 platforms are available for download from the repository's [Releases](https://github.com/qiskit-community/qrmi/releases/latest) tab.

To install from source, follow the instructions in the [documentation](https://github.com/qiskit-community/qrmi/blob/main/INSTALL.md).

## Getting Started

You can explore the full range of example code [here](https://github.com/qiskit-community/qrmi/blob/main/examples). Each example directory includes a README containing more details.

One example of QRMI usage in a compute infrastructure project is the Slurm plugin for quantum resources. QRMI is used in these Slurm plugins to control quantum resources during the lifecycle of a Slurm job. You can find full details of the implementation of Quantum SPANK plugins for Slurm [here](https://github.com/qiskit-community/spank-plugins).

## How to Cite this Work

QRMI is the work of [many people](https://github.com/qiskit-community/qrmi/graphs/contributors) who contribute
to the project at different levels. If you use QRMI, please consider citing the associated overview paper, ["Quantum resources in resource management systems"](https://arxiv.org/abs/2506.10052). This helps support the continued development and visibility of the repository. The BibTeX citation handle can be found in the [BibTeX](CITATION.bib) file.

> [!NOTE]
> We expect multiple versions of the overview paper to be released as the project evolves.

## Contribution Guidelines

If you'd like to contribute to QRMI, please take a look at our
[contribution guidelines](https://github.com/qiskit-community/qrmi/blob/main/CONTRIBUTING.md).

By participating, you are expected to uphold our [code of conduct](https://github.com/qiskit-community/qrmi/blob/main/CODE_OF_CONDUCT.md).

We use [GitHub issues](https://github.com/qiskit-community/qrmi/issues) for tracking requests and bugs.

For further questions, comments and discussion, please consider [joining the Qiskit Slack community](https://qisk.it/join-slack).

## Frequently Asked Questions

A list of frequently asked questions is maintained [here](https://github.com/qiskit-community/qrmi/blob/main/FAQ.md).

## References and Acknowledgements

1. Quantum SPANK plugins for Slurm https://github.com/qiskit-community/spank-plugins
2. Slurm Documentation https://slurm.schedmd.com/
3. Qiskit https://www.ibm.com/quantum/qiskit
4. IBM Quantum https://www.ibm.com/quantum
5. Pasqal https://pasqal.com
6. STFC The Hartree Centre, https://www.hartree.stfc.ac.uk. This work was supported by the Hartree National Centre for Digital Innovation (HNCDI) programme.
7. Rensselaer Polytechnic Institute, Center for Computational Innovation, http://cci.rpi.edu/
8. Alice & Bob https://alice-bob.com/
