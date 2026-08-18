.. _testing:

Testing QRMI
============

.. contents::
   :local:
   :depth: 2

Testing helps ensure QRMI remains reliable, maintainable, and portable across
the different backends and platforms it supports. Contributors are expected to
provide appropriate automated tests for new functionality and bug fixes.

QRMI uses language-specific testing frameworks for its Python, Rust, and C
components. Tests are generally divided into unit tests, which validate
individual components in isolation, and integration tests, which verify
interactions between multiple components, services, or backends.


Rust Testing
------------

QRMI uses Rust's built-in testing framework. All Rust tests are located alongside the Rust crate under:

``src/``

Rust Unit Tests
~~~~~~~~~~~~~~~

Unit tests should be placed close to the code they exercise using
``#[cfg(test)]`` modules.

Example:

::

   src/
   ├── backend.rs
   ├── provider.rs
   └── ...

Within a source file:

.. code-block:: rust

   #[cfg(test)]
   mod tests {
      use super::*;

      #[test]
      fn test_example() {
         // ...
      }
   }

Rust Integration Tests
~~~~~~~~~~~~~~~~~~~~~~

Integration tests should be placed under:

::

   tests/

These tests exercise the crate through its public API in the same way as
an external consumer.

Writing Rust Tests
~~~~~~~~~~~~~~~~~~

-  Keep unit tests close to the implementation where possible.
-  Use integration tests to validate public interfaces and multi-component
   behaviour.

Contributors are encouraged to:

-  Add tests for all new functionality.
-  Cover both success and failure paths.
-  Include regression tests for reported bugs.

Running Rust Tests
~~~~~~~~~~~~~~~~~~

Rust built-in test framework can be executed using the ``cargo test`` command in the terminal:

.. code-block:: shell-session

   cargo test

Python Testing
--------------

QRMI uses the Pytest testing framework. All Python tests can be located under:

``python/tests/``

We separate unit and integration tests:

::

   python/
   ├── qrmi/
   │   └── ...
   └── tests/
      ├── unit/
      └── integration/

Python Unit Tests
~~~~~~~~~~~~~~~~~

QRMI's unit tests follow these three core principles:

-  Be fast.
-  Use no external services.
-  Follow the source tree after ``python/qrmi``.

Example:

`python/tests/unit/pulser/test_connection.py`_

.. _python/tests/unit/pulser/test_connection.py : https://github.com/qiskit-community/qrmi/blob/main/python/tests/unit/pulser/test_connection.py

Following these principles allows for:

-  Logical grouping per vendor or framework.
-  Local ``conftest.py`` files per submodule when needed.
-  Vendor-specific utilities without cross-contamination.

Python Integration Tests
~~~~~~~~~~~~~~~~~~~~~~~~

.. -  May require network access, services, or real backends.

TBD

Running Python Tests
~~~~~~~~~~~~~~~~~~~~

Python tests can be executed using the ``pytest`` command in the terminal:

.. code-block:: shell-session

   pytest

Writing Python Tests
~~~~~~~~~~~~~~~~~~~~

-  File names must follow ``test_*.py`` to be discoverable.
-  Tests should be deterministic, set local seeds as required.

Contributors are encouraged to:

-  Add tests alongside their features.
-  Replicate the source structure under ``unit/`` where appropriate.
-  Introduce vendor-specific fixtures inside scoped directories.

C Testing
---------

QRMI currently exposes a C API through ``qrmi.h``. Dedicated C API tests are
not yet implemented. Contributors adding or modifying the C API are encouraged
to add an accompanying C testing framework and automated test coverage.