.. Quantum Resource Management Interface (QRMI) documentation master file, created by
   sphinx-quickstart on Thu Jul  2 13:18:35 2026.
   You can adapt this file completely to your liking, but it should at least
   contain the root `toctree` directive.

:layout: landing

Quantum Resource Management Interface (QRMI)
============================================

.. rst-class:: lead

    A thin and vendor agnostic layer to access, control and monitor underlying on-prem or cloud quantum computers.

.. container:: buttons

    :doc:`Docs <getting_started/INSTALLATION>`
    `GitHub <https://github.com/qiskit-community/qrmi>`_

.. rubric:: Supported Vendors
   :class: centered

.. grid:: 1 2 4 4

    .. grid-item::

        .. image:: /_static/images/ibm-quantum-logo.png
           :width: 90%
           :target: https://www.ibm.com/quantum
           :align: center

    .. grid-item::

        .. image:: /_static/images/pasqal-logo.png
           :width: 70%
           :target: https://www.pasqal.com/
           :align: center

    .. grid-item::

        .. image:: /_static/images/alice-and-bob-logo.png
           :width: 100%
           :target: https://alice-bob.com/
           :align: center

    .. grid-item::

        .. image:: /_static/images/iqm-logo.png
           :width: 50%
           :target: https://iqm.tech/
           :align: center

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
more information, read the documentation available :ref:`here <task_runner>`.

.. grid:: 1 1 2 3
   :gutter: 2
   :padding: 0
   :class-row: surface

   .. grid-item-card:: :octicon:`mortar-board` Getting Started
      :link: getting_started/INSTALLATION
      :link-type: doc

      The user documentation provides information on how to install and
      use QRMI.

   .. grid-item-card:: :octicon:`tools` Development
      :link: development/CONTRIBUTING
      :link-type: doc

      The developer documentation provides information on how to contribute
      to QRMI and how to run tests.

   .. grid-item-card:: :octicon:`file-code` Examples
      :link: examples/index
      :link-type: doc

      The examples provide information on how to use QRMI with different
      vendor frameworks.

Contributors
------------

.. container:: rounded-image

   .. contributors:: qiskit-community/qrmi
      :avatars:
      :exclude: dependabot[bot], pre-commit-ci[bot]

.. toctree::
   :maxdepth: 2
   :caption: Getting Started
   :hidden:
   
   getting_started/INSTALLATION

.. toctree::
   :maxdepth: 2
   :caption: Additional Resources
   :hidden:
   
   additional_resources/CODE_OF_CONDUCT
   additional_resources/TROUBLESHOOTING

.. toctree::
   :maxdepth: 2
   :caption: Development
   :hidden:
   
   development/CONTRIBUTING
   development/TESTING
   development/DOCUMENTATION
   development/PYTHON_API

.. toctree::
   :maxdepth: 2
   :caption: Examples
   :hidden:
   
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
