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

Prerequisites
-------------

If you are new to contributing to Qiskit, we recommend you do the following
before diving into the code:

-  Read the `Code of Conduct`_

.. _Code of Conduct: https://github.com/qiskit-community/qrmi/blob/main/CODE_OF_CONDUCT.md

-  Familiarize yourself with the Qiskit community (via
   `Slack`_, `GitHub`_, etc.)

.. _Slack: https://qisk.it/join-slack
.. _GitHub: https://github.com/qiskit-community/feedback/discussions

Set up a virtual development environment
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

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

      .. code:: bash

         python3 -m venv ~/.venvs/qrmi-dev

      Activate the environment by invoking the appropriate activation script
      for your system, which can be found within the environment folder. For
      example, for bash/zsh:

      .. code:: bash

         source ~/.venvs/qrmi-dev/bin/activate

      Upgrade ``pip`` within the environment to ensure QRMI dependencies installed
      in the subsequent sections can be located for your system. You need
      ``pip>=25.1`` to use the ``--group`` feature, used to manage developer
      dependency groups:

      .. code:: bash

         pip install -U pip

      You can easily install all the standard developer dependencies for
      in-place testing, documentation-building, and linting using:

      .. code:: bash

         pip install -r requirements-dev.txt

   .. tab:: Conda

      For Conda users, a new environment can be created as follows:

      .. code:: bash

         conda create -y -n QRMIDevenv python=3
         conda activate QRMIDevenv

      Install the QRMI dependencies:

      .. code:: bash

         pip install -e .

Install QRMI from source
~~~~~~~~~~~~~~~~~~~~~~~~

Refer to :ref:`install`.

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

Before marking your PR as “ready for review”, make sure you
have followed the PR checklist below. PRs that adhere to this list are
more likely to be reviewed and merged in a timely manner.

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
#. If you are of the opinion that the modifications you made warrant
   additional tests, feel free to include them
#. Ensure that if your change has an end user facing impact (new
   feature, deprecation, removal etc) that you have added a reno release
   note for that change and that the PR is tagged for the changelog.
#. All contributors have signed the CLA.
#. The PR has a concise and explanatory title (e.g. ``Fixes Issue1234``
   is a bad title!).
#. If the PR addresses an open issue the PR description includes the
   ``fixes #issue-number`` syntax to link the PR to that issue (**you
   must use the exact phrasing in order for GitHub to automatically
   close the issue when the PR merges**)

Pre-commit detect-secrets
~~~~~~~~~~~~~~~~~~~~~~~~~

``detect-secrets`` is an open-source, developer-friendly tool designed
to scan codebases for mistakenly committed secrets—such as API keys,
passwords, and private tokens—before they leak. To keep our credentials
secure, we recommend that all developers integrate this into their
workflow using the following instructions.

-  Prerequisites: Before you begin, ensure you have a Python virtual
   environment (venv) active. You will need to install pre-commit, which
   manages the hooks that run detect-secrets automatically.

.. code:: bash

   pip install pre-commit
   pre-commit install

Please find ``.pre-commit-config.yaml`` for the initial setup. Following
command was used to generate ``.secrets.baseline`` and to maximize the
detection coverage.

.. code:: bash

   detect-secrets scan --force-use-all-plugins > .secrets.baseline

**Handling False Positives** If the pre-commit hook identifies a secret
that you have verified is not sensitive (a false positive), please use
the following command to audit and update the baseline file. Once
updated, include the modified .secrets.baseline in your Pull Request to
ensure the pre-commit passes in the future.

.. code:: bash

   pip install detect-secrets
   detect-secrets scan --force-use-all-plugins --exclude-files '.secrets.*' --exclude-files '.git*' --baseline .secrets.baseline
   detect-secrets audit .secrets.baseline

**Manual Execution and Overrides** To manually trigger a scan of all
files in the repository for a local sanity check, execute the following
command:

.. code:: bash

   pre-commit run --all-files

**Bypassing the Hook (Not Recommended)** While not recommended, if you
must force a commit without running the pre-commit checks (e.g., during
an emergency fix), you may use the ``--no-verify`` flag:

.. code:: bash

   git commit -m "Your message" --no-verify

Code Review
~~~~~~~~~~~

Code review is done in the open and is open to anyone. While only
maintainers have access to merge commits, community feedback on pull
requests is extremely valuable. It is also a good mechanism to learn
about the code base.

Response times may vary for your PR, it is not unusual to wait a few
weeks for a maintainer to review your work, due to other internal
commitments. If you have been waiting over a week for a review on your
PR feel free to tag the relevant maintainer in a comment to politely
remind them to review your work.

Please be patient! Maintainers have a number of other priorities to
focus on and so it may take some time for your work to get reviewed and
merged. PRs that are in a good shape (i.e. following the `Pull request
checklist <#pull-request-checklist>`__) are easier for maintainers to
review and more likely to get merged in a timely manner. Please also
make sure to always be kind and respectful in your interactions with
maintainers and other contributors, you can read `the QRMI Code of
Conduct <https://github.com/qiskit-community/qrmi/blob/main/CODE_OF_CONDUCT.md>`__.

Contributor Licensing Agreement
-------------------------------

Before you can submit any code, all contributors must sign a contributor
license agreement (CLA). By signing a CLA, you’re attesting that you are
the author of the contribution, and that you’re freely contributing it
under the terms of the Apache-2.0 license.

When you contribute to the Qiskit project with a new pull request, a bot
will evaluate whether you have signed the CLA. If required, the bot will
comment on the pull request, including a link to accept the agreement.
The `individual CLA <https://qisk.it/cla>`__ document is available for
review as a PDF.

Note: If your contribution is part of your employment or your
contribution is the property of your employer, then you will more than
likely need to sign a `corporate CLA <https://qisk.it/corporate-cla>`__
too and email it to us at qiskit@us.ibm.com.

Testing
-------

Once you’ve made a code change, it is important to verify that your
change does not break any existing tests and that any new tests that
you’ve added also run successfully. Before you open a new pull request
for your change, you’ll want to run QRMI’s Python test suite (as well as
its Rust-based unit tests if you’ve modified native code).

Testing Python modules
~~~~~~~~~~~~~~~~~~~~~~

The easiest way to run QRMI’s Python test suite is to use
`pytest <https://docs.pytest.org/en/stable/>`__. You can install pytest
with pip: ``pip install -U pytest``.

To run QRMI’s Python test suite:

.. code:: bash

   pip install "$(ls ./target/wheels/qrmi-*.whl)[ibm,pasqal]"
   pytest .

Testing Rust components
~~~~~~~~~~~~~~~~~~~~~~~

Many of QRMI’s core data structures and code are implemented in Rust.

To provide Rust unit testing, we use ``cargo test``. Rust tests are
integrated directly into the Rust file being tested within a ``tests``
module. Functions decorated with ``#[test]`` within these modules are
built and run as tests.

.. code:: rust

   #[cfg(test)]
   mod tests {
       #[test]
       fn my_first_test() {
           assert_eq!(2, 1 + 1);
       }
   }

For more detailed guidance on how to write Rust tests, you can refer to
the Rust documentation’s `guide on writing
tests <https://doc.rust-lang.org/book/ch11-01-writing-tests.html>`__.

For executing the tests with the makefile, a ``make test`` target is
available. The execution of the tests (both via the make target and
during manual invocation) takes into account the ``LOG_LEVEL``
environment variable.

Unsafe code and Miri
~~~~~~~~~~~~~~~~~~~~

Any ``unsafe`` code added to the Rust logic should be exercised by
Rust-space tests, in addition to the more complete Python test suite. In
CI, we run the Rust test suite under
`Miri <https://github.com/rust-lang/miri>`__ as an undefined-behavior
sanitizer.

Miri is currently only available on ``nightly`` Rust channels, so to run
it locally you will need to ensure you have that channel available, such
as by

.. code:: bash

   rustup install nightly --components miri

After this, you can run the Miri test suite with

.. code:: bash

   MIRIFLAGS="<flags go here>" cargo +nightly miri test

For the current set of ``MIRIFLAGS`` used by Qiskit’s CI, see the
```miri.yml`` <https://github.com/Qiskit/qiskit/blob/main/.github/workflows/miri.yml>`__
GitHub Action file. This same file may also include patches to
dependencies to make them compatible with Miri, which you would need to
temporarily apply as well.

Testing the C API
~~~~~~~~~~~~~~~~~

TBD

Writing C API tests
^^^^^^^^^^^^^^^^^^^

TBD

Style and lint
--------------

QRMI uses three tools for Python code formatting and lint checking. The
first tool is `black <https://github.com/psf/black>`__ which is a code
formatting tool that will automatically update the code formatting to a
consistent style. The second tool is
`pylint <https://docs.pytest.org/en/stable/>`__ which is a code linter
which does a deeper analysis of the Python code to find both style
issues and potential bugs and other common issues in Python.

Rust style and lint
~~~~~~~~~~~~~~~~~~~

For formatting and lint checking Rust code, you’ll need to use different
tools than you would for Python. QRMI uses
`rustfmt <https://github.com/rust-lang/rustfmt>`__ for code formatting.
You can simply run ``cargo fmt`` (if you installed Rust with the default
settings using ``rustup``), and it will update the code formatting
automatically to conform to the style guidelines. For lint checking,
QRMI uses `clippy <https://github.com/rust-lang/rust-clippy>`__ which
can be invoked via ``cargo clippy``.

C style and lint
~~~~~~~~~~~~~~~~

QRMI uses
`clang-format <https://clang.llvm.org/docs/ClangFormat.html>`__ to
format C code. The style is based on LLVM, with a few QRMI-specific
adjustments.

Building API docs locally
-------------------------

Prerequisites
~~~~~~~~~~~~~

-  Doxygen (for generating C API document)

   -  ``dnf install doxygen`` for Linux(RHEL/CentOS/Rocky Linux etc)
   -  ``apt install doxygen`` for Linux(Ubuntu etc.)
   -  ``brew install doxygen``\ for MacOS

How to generate Rust API document
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. code:: shell-session

   . ~/.cargo/env
   cargo doc --no-deps --open

How to generate Python API document
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Prerequisites: QRMI Python package is installed in your python virtual
environment(e.g. ``~/py312_qrmi_venv``)

.. code:: shell-session

   source ~/py312_qrmi_venv/bin/activate
   python -m pydoc -p 8290
   Server ready at http://localhost:8290/
   Server commands: [b]rowser, [q]uit
   server> b

Open the following page in your browser.

.. code:: shell-session

   http://localhost:8290/qrmi.html

Quit server.

.. code:: shell-session

   server> q

How to generate C API document
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. code:: shell-session

   doxygen Doxyfile

HTML document will be created under ``./html`` directory. Open
``html/index.html`` in your web browser.

Updating files for new release
------------------------------

To create a new release, the following files must be updated:

-  ``Cargo.toml``

.. code:: toml

     [package]
     name = "qrmi"
     version = "0.14.1"

-  ``Cargo.lock``

.. code:: toml

     [[package]]
     name = "qrmi"
     version = "0.14.1"

-  ``cbindgen.toml``

.. code:: toml

     #define QRMI_VERSION_MAJOR 0
     #define QRMI_VERSION_MINOR 14
     #define QRMI_VERSION_PATCH 1
