.. _faq:

Frequently Asked Questions (FAQ)
================================

.. rst-class:: lead

    Answers to common questions about installing, configuring, using, and developing QRMI.

--------------

.. contents::
   :local:
   :depth: 2

--------------


General Questions
-----------------

What is QRMI?
~~~~~~~~~~~~~

QRMI (:ref:`Quantum Resource Management Interface <qrmi_overview>`) is an open-source,
vendor-neutral software layer that enables high-performance computing
(HPC) systems to access, manage, and monitor quantum computing
resources through a consistent set of APIs.

Why was QRMI created?
~~~~~~~~~~~~~~~~~~~~~

Quantum computing providers often expose different APIs, resource
models, and operational workflows. QRMI reduces this complexity by
providing a common abstraction layer that allows applications and
workload managers to interact with multiple quantum technologies
through a unified interface.

Is QRMI tied to a specific quantum hardware vendor?
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

No. QRMI is designed to be vendor-neutral. It provides a common
interface that can be implemented for different quantum hardware and
service providers, allowing software to interact with multiple
providers without provider-specific integrations.

How does QRMI integrate with HPC systems?
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

QRMI enables quantum devices to be represented as schedulable
resources alongside traditional HPC resources such as CPUs, GPUs,
and storage systems. This simplifies the deployment and management
of hybrid quantum-classical workflows.

Which workload managers are supported?
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

QRMI :ref:`has been demonstrated <qrmi_integrations>` with several workload managers,
including Slurm, PBS, LSF, Grid Engine, Kubernetes, and Flux.

Support for a specific workload manager depends on the integration
being used and the capabilities provided by the local deployment.

What programming languages does QRMI support?
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

QRMI is implemented in Rust and provides APIs for Python, C, and Lua.
These interfaces enable integration with existing HPC tools, services,
and applications across a variety of computing environments.

Is QRMI open source?
~~~~~~~~~~~~~~~~~~~~

Yes. QRMI is an open-source project developed through collaboration
between HPC centres, quantum computing providers, and research
organisations.

Where can I find examples of using QRMI?
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Example applications, integration guides, and reference
implementations are available in the `QRMI GitHub repository`_ and :ref:`project
documentation <examples_index>`.

.. _`QRMI GitHub repository`: https://github.com/qiskit-community/qrmi/tree/main/examples

How can I contribute to QRMI?
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

:ref:`Contributions are welcome <contributing>`. You can contribute by reporting issues,
improving documentation, adding tests, developing new features, or
implementing support for additional quantum providers and HPC
environments.