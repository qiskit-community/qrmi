.. _faq:

Frequently Asked Questions (FAQ)
================================

.. rst-class:: lead

    Answers to common questions about installing, configuring, using, and developing QRMI.

.. contents::
   :local:
   :depth: 2

General Questions
-----------------

What is QRMI?
~~~~~~~~~~~~~

QRMI (Quantum Resource Management Interface) is a vendor-agnostic software layer that enables HPC systems to access, control, and monitor quantum computing resources through a common set of APIs.

Why do we need QRMI?
~~~~~~~~~~~~~~~~~~~~

Quantum providers use different APIs and workflows, making integration complex. QRMI provides a standard interface that reduces development effort and simplifies access to multiple quantum technologies.

Is QRMI tied to a specific quantum hardware vendor?
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

No. QRMI is designed to be vendor-agnostic, allowing applications and schedulers to interact with different quantum systems through the same interface.

How does QRMI integrate with HPC systems?
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

QRMI allows quantum devices to be managed as schedulable resources alongside traditional HPC resources such as CPUs and GPUs, making hybrid quantum-classical workflows easier to deploy.

Which workload managers are supported?
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

QRMI has been demonstrated with a range of workload managers, including Slurm, PBS, LSF, Grid Engine, Kubernetes, and Flux.

What programming languages does QRMI support?
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

QRMI is written in Rust and provides interfaces for Python, C and Lua, allowing it to be integrated into a variety of existing software ecosystems and HPC environments.

Is QRMI open source?
~~~~~~~~~~~~~~~~~~~~

Yes. QRMI is an open-source project developed by a growing community of HPC centres, quantum providers, and research organisations.

