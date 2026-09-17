.. _examples_index:

QRMI Examples
=============

.. rst-class:: lead

    Explore practical examples demonstrating how to use QRMI's core features, integrations, and APIs in real-world workflows.

--------------

QRMI includes a range of examples demonstrating various usages of the software and its associated tools.

Whilst QRMI does not require a workload manager to function, it was primarily designed to work in tandem with one. 
`Slurm`_ is the most widely used workload manager and was the first of QRMI's integrations. 
All of these examples were written with the Slurm workload manager in mind. 

.. _Slurm: https://slurm.schedmd.com/overview.html

QRMI's existing workload manager integrations include:

- `Slurm SPANK Plugin`_
- `OpenPBS`_
- `LSF`_
- `Grid Engine`_
- `Flux Framework`_

.. _Slurm SPANK Plugin: https://github.com/qiskit-community/spank-plugins
.. _OpenPBS: https://github.com/ohtanim/pbs-hooks-for-qrmi 
.. _LSF: https://github.com/IBM/lsf-quantum 
.. _Grid Engine: https://github.com/hpc-gridware/qpu-resource/ 
.. _Flux Framework: https://github.com/qrmi-community/flux-shell-integration

For more information about QRMI's workload manager integrations, see our related paper, `"Quantum resources in resource management systems"`_.

.. _"Quantum resources in resource management systems": https://arxiv.org/abs/2506.10052

--------------

.. toctree::
    :maxdepth: 2
    
    bindings/index
    pulser/index
    qiskit_primitives/index
    task_runner/index