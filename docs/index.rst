.. Quantum Resource Management Interface (QRMI) documentation master file, created by
   sphinx-quickstart on Thu Jul  2 13:18:35 2026.
   You can adapt this file completely to your liking, but it should at least
   contain the root `toctree` directive.

Quantum Resource Management Interface (QRMI)
============================================

|License| |Current Release| |Platform| |PyPI - Python Version| |Minimum
rustc 1.91| |Downloads| |image1|\  |DOI| |arXiv| |CI|

The *Quantum Resource Management Interface* (QRMI) is a vendor-agnostic
library for high-performance compute (HPC) systems to access, control,
and monitor the behavior of quantum computational resources. It acts as
a thin middleware layer that abstracts away the complexities associated
with controlling quantum resources through a set of simple APIs. Written
in Rust, this interface also exposes Python and C APIs for ease of
integration into nearly any computational environment.

The source code to build and deploy QRMI is available
`here <https://github.com/qiskit-community/qrmi>`__.

An optional ``task_runner`` command line tool to execute quantum
payloads against quantum hardware is included in the Python package. For
more information, read the documentation available
`here <https://github.com/qiskit-community/qrmi/blob/main/python/qrmi/tools/task_runner/README.md>`__.

.. toctree::
   :maxdepth: 2
   :numbered:
   :caption: Table of Contents:
   
   Code of Conduct <CODE_OF_CONDUCT>
   user_docs/index
   developer_docs/index
   Troubleshooting <TROUBLESHOOTING>
   examples/index

.. |License| image:: https://img.shields.io/github/license/qiskit-community/qrmi.svg?
   :target: https://opensource.org/licenses/Apache-2.0
.. |Current Release| image:: https://img.shields.io/github/release/qiskit-community/qrmi.svg?
   :target: https://github.com/qiskit-community/qrmi/releases
.. |Platform| image:: https://img.shields.io/badge/%F0%9F%92%BB_Platform-Linux%20%7C%20macOS-blue
.. |PyPI - Python Version| image:: https://img.shields.io/pypi/pyversions/qrmi
.. |Minimum rustc 1.91| image:: https://img.shields.io/badge/rustc-1.91+-blue.svg
   :target: https://rust-lang.github.io/rfcs/2495-min-rust-version.html
.. |Downloads| image:: https://img.shields.io/pypi/dm/qrmi.svg
   :target: https://pypi.org/project/qrmi/
.. |image1| image:: https://static.pepy.tech/badge/qrmi
   :target: https://pepy.tech/project/qrmi
.. |DOI| image:: https://zenodo.org/badge/DOI/10.5281/zenodo.20650771.svg
   :target: https://doi.org/10.5281/zenodo.20650771
.. |arXiv| image:: https://img.shields.io/badge/arXiv-2506.10052-b31b1b.svg
   :target: https://arxiv.org/abs/2506.10052
.. |CI| image:: https://github.com/qiskit-community/qrmi/actions/workflows/on-schedule.yml/badge.svg
   :target: https://github.com/qiskit-community/qrmi/actions/workflows/on-schedule.yml
