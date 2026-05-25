# Quantum resource management interface (QRMI)

The Quantum resource management interface (QRMI) is a vendor-agnostic library for high-performance compute (HPC) systems to access, control, and monitor the behavior of quantum computational resources. It acts as a thin middleware layer that abstracts away the complexities associated with controlling quantum resources through a set of simple APIs. Written in Rust, this interface also exposes Python and C APIs for ease of integration into nearly any computational environment.

Find the source code to build and deploy QRMI in this [GitHub repository](https://github.com/qiskit-community/qrmi).

An optional task_runner command line tool to execute quantum payloads against quantum hardware is included in the Python package. Find the [full documentation](https://github.com/qiskit-community/qrmi/blob/main/python/qrmi/tools/task_runner/README.md) in the GitHub repository.

## Installation

### Python

For running QRMI on Python we recommend installing QRMI via ``pip``:

```bash
pip install qrmi
```

To include support for specific quantum resources, install with the corresponding extras:

```bash
pip install "qrmi[ibm]"       # IBM quantum resources
pip install "qrmi[iqm]"       # IQM quantum resources
pip install "qrmi[pasqal]"    # Pasqal quantum resources
pip install "qrmi[all]"       # All quantum resources
```

Or combine multiple resources:

```bash
pip install "qrmi[ibm,pasqal]"
```

> [!NOTE]
> Note: `ibm` and `iqm` extras cannot be installed together, as they depend on incompatible versions of Qiskit.

Pip will handle all dependencies automatically and you will always install the latest (and well-tested) version.

To install from source, follow the instructions in the [documentation](https://github.com/qiskit-community/qrmi/blob/main/INSTALL.md).

### Standalone C library

Prebuilt binaries for Linux x86_64, ppc64le, and arm64 platforms are available for download from the [Releases](https://github.com/qiskit-community/qrmi/releases/latest) tab of this GitHub repository.

To install from source, follow the instructions in the [documentation](https://github.com/qiskit-community/qrmi/blob/main/INSTALL.md).


## Getting Started

This GitHub repository contains [a variety of example code](https://github.com/qiskit-community/qrmi/blob/main/examples). See the README in each directory for details.

One of example of usage of QRMI in compute infrastrcture project is Slurm plugin for quantum resources. QRMI is used in Slurm plugin to control quantum resources during lifetime of Slurm job. See implementation and documentation of [Quantum spank plugins for Slurm](https://github.com/qiskit-community/spank-plugins).

## How to Cite This Work

QRMI is the work of [many people](https://github.com/qiskit-community/qrmi/graphs/contributors) who contribute
to the project at different levels. If you use QRMI, please consider citing the associated overview paper  [Quantum resources in resource management systems](https://arxiv.org/abs/2506.10052). This helps support the continued development and visibility of the repository. The BibTeX citation handle can be found in the [BibTeX file](CITATION.bib) file.

Note we expect multiple versions of the overview paper to be released as the project evolves.


## Contribution Guidelines

For information on how to contribute to this project, please take a look at our [contribution guidelines](CONTRIBUTING.md).

If you'd like to contribute to QRMI, please take a look at our
[contribution guidelines](CONTRIBUTING.md). By participating, you are expected to uphold our [code of conduct](CODE_OF_CONDUCT.md).

We use [GitHub issues](https://github.com/qiskit-community/qrmi/issues) for tracking requests and bugs. Please
[join the Qiskit Slack community](https://qisk.it/join-slack) for discussion, comments, and questions.


## References and Acknowledgements
1. Quantum spank plugins for Slurm https://github.com/qiskit-community/spank-plugins
2. Slurm documentation https://slurm.schedmd.com/
3. Qiskit https://www.ibm.com/quantum/qiskit
4. IBM Quantum https://www.ibm.com/quantum
5. Pasqal https://pasqal.com
6. STFC The Hartree Centre, https://www.hartree.stfc.ac.uk. This work was supported by the Hartree National Centre for Digital Innovation (HNCDI) programme.
7. Rensselaer Polytechnic Institute, Center for Computational Innovation, http://cci.rpi.edu/
8. Alice & Bob https://alice-bob.com/
