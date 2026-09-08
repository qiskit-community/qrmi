.. _documentation:

Documentation
=============

.. rst-class:: lead

   Guidance on how QRMI documentation is structured, maintained, and generated for publication.

--------------

.. contents::
   :local:
   :depth: 2

--------------

Overview
--------


Sphinx
~~~~~~

These pages are built using `Sphinx`_, a documentation generator. The process of building these HTML pages from the reStructured Text source files is automated via the Sphinx Documentation GitHub Action.

.. _Sphinx: https://www.sphinx-doc.org/en/master/

The GitHub Action is responsible for two tasks; building the documentation, then deploying it to GitHub Pages. 

- Pushing to a feature branch with an associated PR triggers the ``build`` job, which carries out checks ensuring the documentation builds correctly.
- Pushing to ``main`` triggers both the ``build`` and ``deploy`` jobs, which build the documentation and deploy it to GitHub Pages. 

Further information about Sphinx can be found in the `Sphinx documentation`_.

.. _Sphinx documentation: https://www.sphinx-doc.org/en/master/#user-guide


pandoc
~~~~~~

Sphinx does not support Markdown natively, so any Markdown files must be converted to reStructured Text before they can be included in the documentation.

`pandoc`_ is an open-source tool that can assist with this conversion. Once installed, you can convert a ``.md`` file to an ``.rst`` file, you could do so by running the following command:

.. code-block:: bash

   pandoc -f markdown -t rst -o output.rst input.md

Multiple Markdown files can be converted in bulk:

.. code-block:: bash

   for f in *.md; do
    pandoc -s "$f" -o "${f%.md}.rst"
   done

More advanced information on pandoc usage can be found in the `pandoc documentation`_.

.. _pandoc: https://pandoc.org/
.. _pandoc documentation: https://pandoc.org/MANUAL.html


.. _adding_documentation:

Adding Documentation
--------------------

All documentation files are stored in the ``docs`` directory. The ``index.rst`` file defines the content of the landing page, as well as structure of the documentation (as seen in the sidebar).

Sphinx stores built HTML files in the ``_build`` directory. Static files, such as images, ``.css`` and ``.js`` files, are stored in ``_static`` and its associated subdirectories.

If you would like to add to the existing documentation, follow these steps:

#. Create a new reStructured Text (``.rst``) file in the ``docs`` directory. If the file relates to an existing topic, you can place it in the appropriate subdirectory.

#. In ``docs/index.rst``, add a reference to the new file in the appropriate section of the ``toctree`` directive. For example, for a new file called ``new_topic.rst``:

   .. code-block:: rst
      :caption: new_topic.rst

      .. toctree::
         :maxdepth: 2
         :caption: New Section

         new_topic

#. :ref:`Build the documentation locally. <building_documentation>` If the build is successful, commit and push your changes to the repository.


Local Documentation
-------------------

.. _building_documentation:

Building Local Documentation
~~~~~~~~~~~~~~~~~~~~~~~~~~~~

To build, test and verify the changes locally (and identify any errors):

#. Install the required dependencies for building the documentation:

   .. code-block:: bash

      python -m pip install --upgrade pip
      pip install -e ".[all, docs]"

#. Build the documentation and serve it locally:

   .. code-block:: bash

      cd docs/
      sphinx-autobuild . _build/html/

The build process will identify any errors, such as missing references, toctree issues or syntax errors. 


.. _api_refs:

API References
~~~~~~~~~~~~~~

QRMI's hosted API references can be accessed using the below site links:

- :ref:`rust_api`
- :ref:`python_api`
- :ref:`c_api`
- :ref:`lua_api`

To build the API references locally, follow the instructions below.


Prerequisites
^^^^^^^^^^^^^

-  Doxygen (for generating C API document)

   -  ``dnf install doxygen`` for Linux(RHEL/CentOS/Rocky Linux etc)
   -  ``apt install doxygen`` for Linux(Ubuntu etc.)
   -  ``brew install doxygen`` for MacOS


Building Local API References
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

.. tabs::

   .. tab:: Rust API

      1. Build the Rust API docs using the following command:
      
      .. code-block:: bash

         . ~/.cargo/env
         cargo doc --no-deps --open

   .. tab:: C API

      1. Build the C API docs using the following command:

      .. code-block:: bash

         doxygen Doxyfile

      By default, the HTML documents will be created in the ``build/doxygen/html/``
      directory. 
      
      2. Open ``build/doxygen/html/index.html`` in your web browser.

   .. tab:: Python API

      .. important:: 
         
            Ensure the QRMI Python package is installed in your Python virtual
            environment (e.g. ``~/py312_qrmi_venv``).

      1. Build the Python API docs using the following command:

      .. code-block:: bash

         source ~/py312_qrmi_venv/bin/activate
         python -m pydoc -p 8290
         Server ready at http://localhost:8290/
         Server commands: [b]rowser, [q]uit
         server> b

      2. Navigate to the following address in your browser:

      .. code-block:: bash

         http://localhost:8290/qrmi.html

      3. Quit the server:

      .. code-block:: bash

         server> q

Hosting the Sphinx API References Locally
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

These API references are automatically built as part of QRMI's CI/CD pipeline and aren't automatically available when the repository is cloned. Building the API references locally and moving them into the appropriate subdirectories will allow you to access them via your local build of the documentation.

.. tabs::

   .. tab:: Rust API

      1. Build the Rust API docs using the following command:

      .. code-block:: bash
         
         . ~/.cargo/env
         cargo doc --no-deps

      2. Copy the generated documentation into the appropriate subdirectory:

      .. code-block:: bash
         
         mkdir -p docs/_build/html/rust
         cp -R target/doc/* docs/_build/html/rust/

      3. Build the documentation and serve it locally:

      .. code-block:: bash

         cd docs/
         sphinx-autobuild . _build/html/

   .. tab:: C API

      1. Install Doxygen using the following command:

      .. code-block:: bash

         sudo apt-get update
         sudo apt-get install -y doxygen

      2. Build the C API docs using the following command:

      .. code-block:: bash

         mkdir -p build/doxygen
         doxygen Doxyfile

      3. Copy the generated documentation into the appropriate subdirectory:

      .. code-block:: bash

         mkdir -p docs/_build/html/c
         cp -R build/doxygen/html/* docs/_build/html/c/

      4. Build the documentation and serve it locally:

      .. code-block:: bash

         cd docs/
         sphinx-autobuild . _build/html/

   .. tab:: Python API

      1. Build the Python API docs using the following command:

      .. code-block:: bash

         sphinx-apidoc --no-toc -o docs/_api/python/ python/qrmi/

      2. Build the documentation and serve it locally:

      .. code-block:: bash

         cd docs/
         sphinx-autobuild . _build/html/


Theming and Customisation
-------------------------

Theming and customisation (such as extensions, HTML options, etc.) are configured in ``docs/conf.py``. 

This documentation uses the `Shibuya`_ theme. 

.. _Shibuya: https://shibuya.lepture.com/
