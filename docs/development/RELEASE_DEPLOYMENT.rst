.. _release_deployment:

Release & Deployment
====================

.. rst-class:: lead

   Guidance for releasing and deploying QRMI.

--------------

.. contents::
   :local:
   :depth: 2

--------------

Creating a new release
----------------------

To create a new release, the following files must be updated:

-  ``Cargo.toml``

.. code-block:: toml

     [package]
     name = "qrmi"
     version = "0.14.1"

-  ``Cargo.lock``

.. code-block:: toml

     [[package]]
     name = "qrmi"
     version = "0.14.1"

-  ``cbindgen.toml``

.. code-block:: toml

     #define QRMI_VERSION_MAJOR 0
     #define QRMI_VERSION_MINOR 14
     #define QRMI_VERSION_PATCH 1


How can I check which version of QRMI is linked into a binary?
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. note::

    Linux only.

Every build of QRMI embeds its own crate version and git commit hash
directly into the compiled artifact (shared library, static library, or
any binary that links it), so you can check it without running the code.

Using ``strings``
^^^^^^^^^^^^^^^^^

.. code:: bash

   strings /path/to/libqrmi.so | grep QRMI_BUILD_VERSION
   QRMI_BUILD_VERSION:0.24.0;QRMI_GIT_HASH:0dac1793b013

Using ``readelf``
^^^^^^^^^^^^^^^^^

.. code:: bash

   readelf -p .version_info /path/to/libqrmi.so

   String dump of section '.version_info':
     [    4f]  QRMI_BUILD_VERSION:0.24.0;QRMI_GIT_HASH:0dac1793b013

(The exact offset shown after ``[ ]`` will vary depending on what else
is linked into the same ``.version_info`` section — see below.)

What each field means
^^^^^^^^^^^^^^^^^^^^^

+-----------------------------------+-----------------------------------+
| Field                             | Meaning                           |
+===================================+===================================+
| ``QRMI_BUILD_VERSION``            | QRMI's crate version, from        |
|                                   | ``CARGO_PKG_VERSION`` (i.e. the   |
|                                   | ``version`` field in              |
|                                   | ``Cargo.toml``) at the time it    |
|                                   | was built.                        |
+-----------------------------------+-----------------------------------+
| ``QRMI_GIT_HASH``                 | The exact git commit QRMI was     |
|                                   | built from, resolved via          |
|                                   | ``git rev-parse --short=12 HEAD`` |
|                                   | in ``build.rs``. Useful when the  |
|                                   | consuming project pins a moving   |
|                                   | branch (e.g. ``main``) rather     |
|                                   | than a fixed release tag.         |
+-----------------------------------+-----------------------------------+

This is especially useful when diagnosing issues caused by a version
mismatch between a deployed binary that links QRMI (such as the
`SPANK Plugins for Slurm`_ Slurm plugin) and the QRMI Python package used by
client workloads — you can confirm exactly which QRMI build is present
without rebuilding or adding logging.

.. _SPANK Plugins for Slurm: https://github.com/qiskit-community/spank-plugins


If you're looking at ``spank_qrmi.so`` specifically
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

``spank_qrmi.so`` embeds its own version/git-hash marker alongside
QRMI's, in the same ``.version_info`` section:

.. code:: bash

   strings /path/to/spank_qrmi.so | grep -E "SPANK_QRMI|QRMI_BUILD"
   SPANK_QRMI_BUILD_VERSION=0.11.0;SPANK_QRMI_GIT_HASH=0dac1793b013
   QRMI_BUILD_VERSION:0.24.0;QRMI_GIT_HASH:0dac1793b013

   readelf -p .version_info /path/to/spank_qrmi.so
   String dump of section '.version_info':
     [     0]  SPANK_QRMI_BUILD_VERSION=0.11.0;SPANK_QRMI_GIT_HASH=3845dd911381
     [    3b]  QRMI_BUILD_VERSION:0.24.0;QRMI_GIT_HASH:7a9573703ac4

The two are independent: ``SPANK_QRMI_*`` describes the plugin binary
itself, ``QRMI_BUILD_VERSION``/``QRMI_GIT_HASH`` describes the QRMI
crate it was linked against. See the SPANK Plugin's own
`FAQ`_ for details on the former.

.. _FAQ: https://github.com/qiskit-community/spank-plugins/blob/main/docs/FAQ.md


Additional Notes
^^^^^^^^^^^^^^^^

-  If ``QRMI_GIT_HASH`` shows ``unknown``, QRMI was most likely built
   from a source tree without a ``.git`` directory (e.g. an extracted
   release tarball, or a local checkout pointed at via ``QRMI_ROOT`` in
   the consuming project's build).
-  If neither ``strings`` nor ``readelf`` show a ``.version_info``
   section (or it only shows a ``SPANK_QRMI_*`` entry with no
   ``QRMI_BUILD_VERSION``), the binary may have been built before this
   feature was introduced, or the section may have been removed by a
   full ``strip -s`` pass in the deployment pipeline. Re-run
   ``strip --strip-debug`` instead, or add
   ``--keep-section=.version_info`` to the ``strip``/``objcopy``
   invocation, to preserve it.
-  This works the same way regardless of whether QRMI was linked as a
   shared library (``cdylib``) or a static library (``staticlib``) — the
   marker survives static linking as long as at least one other QRMI
   symbol is referenced by the final binary.
