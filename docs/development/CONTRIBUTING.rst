.. _contributing:

Contributing to QRMI
====================

.. rst-class:: lead

QRMI is an open-source project committed to bringing quantum computing
to people of all backgrounds. This page describes how you can join the
QRMI community in this goal.

----------------

.. contents::
   :local:
   :depth: 2

.. _contributing_prereq:

Prerequisites
-------------

If you are new to contributing to Qiskit, we recommend you do the following
before diving into the code:

-  Read the :ref:`Code of Conduct <code_of_conduct>`

-  Familiarise yourself with the Qiskit community (via
   `Slack`_, `GitHub`_, etc.)

.. _Slack: https://qisk.it/join-slack
.. _GitHub: https://github.com/qiskit-community/feedback/discussions

.. _contributing_cla:

Contributor Licensing Agreement
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

- `Individual CLA`_
- `Corporate CLA`_

.. _Individual CLA: https://qisk.it/cla
.. _Corporate CLA: https://qisk.it/corporate-cla

Before you can submit any code, all contributors must sign a **contributor
license agreement** (CLA). By signing a CLA, you are attesting that you are
the author of the contribution, and that you're freely contributing it
under the terms of the Apache-2.0 license.

When you contribute to the Qiskit project with a new pull request, a bot
will evaluate whether you have signed the CLA. If required, the bot will
comment on the pull request, including a link to accept the agreement.
The `Individual CLA`_ document is available for review as a PDF.

.. note::

   If your contribution is part of your employment or your contribution is the 
   property of your employer, then you likely need to sign a `Corporate CLA`_ too.
   Please email this to us at qiskit@us.ibm.com.

Setting up the developer environment
------------------------------------

Create a virtual environment
~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Virtual environments are used for QRMI development to isolate the
development environment from system-wide packages. This way, we avoid
inadvertently becoming dependent on a particular system configuration.
For developers, this also makes it easy to maintain multiple
environments (e.g. one per supported Python version, for older versions
of QRMI, etc.).

.. tabs::

   .. tab:: Python

      All Python versions supported by Qiskit include a built-in virtual
      environment module, `venv`_.

      .. _venv: https://docs.python.org/3/tutorial/venv.html

      Start by creating a new virtual environment with ``venv``. The resulting
      environment will use the same version of Python that created it and will
      not inherit installed system-wide packages by default. The specified
      folder will be created and is used to hold the environment's
      installation. It can be placed anywhere. For more details, see the
      official Python documentation, `Creation of virtual environments`_.

      .. _Creation of virtual environments: https://docs.python.org/3/library/venv.html

      .. code-block:: bash

         python3 -m venv ~/.venvs/qrmi-dev

      Activate the environment by invoking the appropriate activation script
      for your system, which can be found within the environment folder. For
      example, for bash/zsh:

      .. code-block:: bash

         source ~/.venvs/qrmi-dev/bin/activate

      Upgrade ``pip`` within the environment to ensure QRMI dependencies installed
      in the subsequent sections can be located for your system. You need
      ``pip>=25.1`` to use the ``--group`` feature, used to manage developer
      dependency groups:

      .. code-block:: bash

         pip install -U pip

      You can easily install all the standard developer dependencies for
      in-place testing, documentation-building, and linting using:

      .. code-block:: bash

         pip install -r requirements-dev.txt

   .. tab:: Conda

      For Conda users, a new environment can be created as follows:

      .. code-block:: bash

         conda create -y -n QRMIDevenv python=3
         conda activate QRMIDevenv

      Install the QRMI dependencies:

      .. code-block:: bash

         pip install -e .

Install QRMI from source
~~~~~~~~~~~~~~~~~~~~~~~~

Refer to :ref:`install`.

.. _contributing_issues:

Issues and pull requests
------------------------

We use `GitHub pull requests`_ to accept contributions.

.. _GitHub pull requests: https://docs.github.com/en/pull-requests

While not required, it is best practice to **open a new issue for bug fixes 
and feature development**, before opening a pull request. This allows
discussion with the community about your work:

.. important::

   Issues provide a place to talk about the idea and how we can work together
   to implement it in the code. They let the community know what you are
   working on and offer help and feedback. Issues are numbered and can be
   referenced during discussions with other community and team members.

If you've written some code but need help finishing it, want to get
initial feedback on it prior to finishing it, or want to share it and
discuss prior to finishing the implementation, you can open a **Draft
pull request** and prepend the title with the **[WIP]** tag (for Work In
Progress). This indicates to reviewers that the code in the PR isn't final
and will change. The branch will not be merged until it is finished.
You, or a reviewer, can remove the [WIP] tag when the code is ready to be
reviewed for merging.

Before marking your PR as "ready for review", make sure you
have followed the PR checklist below. PRs that adhere to this list are
more likely to be reviewed and merged in a timely manner.

.. _pull_request_checklist:

Pull request checklist
~~~~~~~~~~~~~~~~~~~~~~

When submitting a pull request for review, please ensure that:

#. The code follows the **code style** of the project and successfully
   passes the **CI tests**. For convenience, you can execute the following
   commands locally, which will run these checks and report any issues.

   -  ``make lint-rust-all``
   -  ``make lint-wheels``
   -  ``make fmt-rust``
   -  ``make fmt-python``

#. The **documentation has been updated** accordingly. In particular, if a
   function or class has been modified during the PR, please update the
   **docstring** accordingly.

#. If your changes warrant additional tests, feel free to include them.

#. Ensure that if your change impacts the end-user (new feature, deprecation, 
   removal, etc.) that you have added a Reno release note for that change and that
   the PR is tagged for the changelog.

#. Ensure all contributors have signed the :ref:`CLA <contributing_cla>`.

#. The PR has a concise and descriptive title.

   - ``Fixes Issue1234`` is a bad title. ``Fix <ERROR_NAME>`` is much more descriptive.

#. If the PR addresses an open issue, the PR description includes the
   ``Fixes #<ISSUE_NUMBER>`` syntax to link the PR to that issue.

   - You must use the **exact phrasing** for GitHub to automatically close the issue when the PR merges.

Pre-commit ``detect-secrets``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

``detect-secrets`` is an open-source, developer-friendly tool designed
to scan codebases for mistakenly committed secrets (such as API keys,
passwords, and private tokens) before they leak. To keep our credentials
secure, we recommend that all developers integrate this into their
workflow using the following instructions:

.. attention::
   Before you begin, ensure you have a **Python virtual environment** (i.e. ``venv``) active. You will need to install ``pre-commit``, which manages the hooks that run ``detect-secrets`` automatically.

Installing ``pre-commit``
^^^^^^^^^^^^^^^^^^^^^^^^^

#. Run the following commands in your virtual environment terminal:

.. code-block:: bash

   pip install pre-commit
   pre-commit install

Find ``.pre-commit-config.yaml`` for the initial setup. 

#. Run the following command to generate a ``.secrets.baseline`` file.

.. code-block:: bash

   detect-secrets scan --force-use-all-plugins > .secrets.baseline

This file maximises the detection coverage.

Handling False Positives
^^^^^^^^^^^^^^^^^^^^^^^^

If the pre-commit hook identifies a secret that you have verified is not sensitive
(a false positive), please use the following command to audit and update the baseline file.
Once updated, include the modified .secrets.baseline in your PR to ensure the pre-commit passes in the future.

.. code-block:: bash

   pip install detect-secrets
   detect-secrets scan --force-use-all-plugins --exclude-files '.secrets.*' --exclude-files '.git*' --baseline .secrets.baseline
   detect-secrets audit .secrets.baseline

Manual Execution and Overrides
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

To manually trigger a scan of all files in the repository for a local sanity check,
execute the following command:

.. code-block:: bash

   pre-commit run --all-files

Bypassing the Hook
^^^^^^^^^^^^^^^^^^

.. warning:: 

   Bypassing the hook is not recommended.

If you must force a commit without running the pre-commit checks
(e.g. during an emergency fix), you may use the ``--no-verify`` flag:

.. code-block:: bash

   git commit -m "Your message" --no-verify

Code Review
~~~~~~~~~~~

Code review is transparent and open to anyone. While only
maintainers have permission to merge commits, community feedback on pull
requests is extremely valuable. It is also a good way to learn
about the code base.

Response times may vary for your PR. It is not unusual to wait a few
weeks for a maintainer to review your work, due to other internal
commitments. If you have been waiting over a week for a review on your
PR, feel free to tag the relevant maintainer in a comment to politely
remind them to review your work.

Please be patient! Maintainers have a number of other priorities to
focus on, therefore it may take some time for your work to be reviewed and
merged. PRs that are in a good shape (i.e. following the 
:ref:`pull request checklist <pull_request_checklist>`) are easier for maintainers
to review and are more likely to get merged in a timely manner. 

Please also make sure to always be kind and respectful in your interactions with
maintainers and other contributors, in line with the :ref:`QRMI Code of Conduct <code_of_conduct>`.

Documentation
-------------

Please ensure any code changes are reflected in the documentation. 
Refer to :ref:`adding_documentation` for further guidance.

.. _contributing_testing:

Testing
-------

Once you've made a code change, it is important to verify that it doesn't break 
any existing tests and that newly added tests run successfully. Before 
you open a new pull request for your change, run QRMI's Python test suite. If you've
modified native code, you should also run its Rust-based unit tests.

Running Unit Tests
~~~~~~~~~~~~~~~~~~

.. tabs::

   .. tab:: Python

      `pytest`_ offers the easiest way to run QRMI's Python test suite.

      .. _pytest: https://docs.pytest.org/en/stable/

      You can install pytest using ``pip``: 

      .. code-block:: bash
         
         pip install -U pytest

      To run QRMI's Python test suite:

      .. code-block:: bash

         pip install "$(ls ./target/wheels/qrmi-*.whl)[ibm,pasqal]"
         pytest .

   .. tab:: Rust

      Many of QRMI's core data structures and code are implemented in Rust.

      ``cargo test`` is responsible for Rust unit testing. Rust tests are
      integrated directly into the Rust file being tested within a ``tests``
      module. Functions within these modules decorated with ``#[test]`` are
      built and run as tests.

      .. code-block:: rust
         :linenos:

         #[cfg(test)]
         mod tests {
            #[test]
            fn my_first_test() {
               assert_eq!(2, 1 + 1);
            }
         }

      For more detailed information on how to write Rust tests, you can refer to
      the Rust documentation's guidance on `writing tests`_.

      .. _writing tests: https://doc.rust-lang.org/book/ch11-01-writing-tests.html

      To execute the tests with the makefile, a ``make test`` target is
      available. The execution of the tests (both via the make target and
      during manual invocation) takes into account the ``LOG_LEVEL``
      environment variable.

.. Unsafe code and Miri
.. ~~~~~~~~~~~~~~~~~~~~

.. Any ``unsafe`` code added to the Rust logic must be executed by
.. Rust-space tests, in addition to the more complete Python test suite. In
.. CI, we run the Rust test suite under `Miri`_ as an undefined-behavior
.. sanitizer.

.. .. _Miri: https://github.com/rust-lang/miri

.. Miri is currently only available on ``nightly`` Rust channels, so to run
.. it locally you will need to ensure you have that channel available:

.. .. code-block:: bash

..    rustup install nightly --components miri

.. After this, you can run the Miri test suite using the following command:

.. .. code-block:: bash

..    MIRIFLAGS="<FLAGS_GO_HERE>" cargo +nightly miri test

.. For the current set of ``MIRIFLAGS`` used by Qiskit's CI, see the
.. `miri.yml`_ GitHub Action file. This same file may also include patches to
.. dependencies to make them compatible with Miri, which you would need to
.. temporarily apply as well.

.. .. _miri.yml: https://github.com/Qiskit/qiskit/blob/main/.github/workflows/miri.yml

Testing the C API
~~~~~~~~~~~~~~~~~

TBD

Writing C API tests
^^^^^^^^^^^^^^^^^^^

TBD

.. _contributing_style:

Style and linting
-----------------

.. tabs::

   .. tab:: Python

      Execute the following commands to fix any formatting issues before submitting a PR.

      .. code-block:: shell-session

         $ source ~/py312_qrmi_venv/bin/activate
         $ cd examples
         $ pylint ./python
         $ black --check ./python

      QRMI uses two tools for Python code formatting and lint checking. The
      first tool is `black`_ which is a code formatting tool that will automatically
      update the code formatting to a consistent style.

      .. _black: https://github.com/psf/black

      The second tool is `pylint`_, which is a code linter capable of deeper
      analysis of the Python code to find both style issues and potential bugs
      and other common issues.

      .. _pylint: https://pypi.org/project/pylint/

   .. tab:: Rust

      Execute the following commands to fix any formatting issues before submitting a PR.

      .. code-block:: shell-session

         $ . ~/.cargo/env
         $ cargo fmt --all -- --check
         $ cargo clippy --all-targets -- -D warnings
         $ cd examples/rust
         $ cargo fmt --all -- --check
         $ cargo clippy --all-targets -- -D warnings

      QRMI uses `rustfmt`_ for Rust formatting and linting. You can run ``cargo fmt``
      (if you installed Rust with the default settings using ``rustup``), and it will
      automatically update the code formatting according to the style guidelines.

      .. _rustfmt: https://github.com/rust-lang/rustfmt

      For lint checking, QRMI uses `clippy`_ which can be invoked using ``cargo clippy``.

      .. _clippy: https://github.com/rust-lang/rust-clippy

   .. tab:: C

      To format C code, QRMI uses `clang-format`_. The style is based on LLVM,
      with a few QRMI-specific adjustments.

      .. _clang-format: https://clang.llvm.org/docs/ClangFormat.html

.. _contributing_release:

Updating files for new release
------------------------------

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
