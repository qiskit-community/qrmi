.. _documentation:

QRMI Documentation
==================

.. rst-class:: lead

QRMI's ongoing development requires a robust and comprehensive documentation system. This section outlines how to add new documentation to the project.

----------------

.. contents::
   :local:
   :depth: 2

Sphinx
------

These pages are built using `Sphinx`_, a documentation generator. The process of building these HTML pages from the reStructured Text source files is automated via a GitHub Action.

.. _Sphinx: https://www.sphinx-doc.org/en/master/

Theming and Customisation
~~~~~~~~~~~~~~~~~~~~~~~~~

Theming and customisation (such as extensions, HTML options, etc.) are configured in ``docs/conf.py``. 

This documentation uses the `Shibuya`_ theme. 


.. _Shibuya: https://shibuya.lepture.com/

.. _adding_documentation:

Adding Documentation
--------------------

All documentation files are stored in the ``docs`` directory. The ``index.rst`` file defines the content of the landing page, as well as structure of the documentation (as seen in the sidebar).

Sphinx stores built HTML files in the ``_build`` directory. Static files, such as images, ``.css`` and ``.js`` files, are stored in ``_static`` and its associated subdirectories.

If you would like to add to the existing documentation, follow these steps:

#. Create a new reStructured Text (`.rst`) file in the ``docs`` directory. If the file relates to an existing topic, you can place it in the appropriate subdirectory.

#. In ``docs/index.rst``, add a reference to the new file in the appropriate section of the ``toctree`` directive. For example, for a new file called ``new_topic.rst``:

   .. code-block:: rst

      .. toctree::
         :maxdepth: 2
         :caption: New Section

         new_topic

#. To test and verify the changes locally (and identify any errors), run the following command:

   .. code-block:: bash

      sphinx-autobuild . _build/

.. _api_docs:

API Documentation
-----------------

QRMI's API documentation can be built locally using the below instructions.

Prerequisites
~~~~~~~~~~~~~

-  Doxygen (for generating C API document)

   -  ``dnf install doxygen`` for Linux(RHEL/CentOS/Rocky Linux etc)
   -  ``apt install doxygen`` for Linux(Ubuntu etc.)
   -  ``brew install doxygen`` for MacOS

.. tabs::

   .. tab:: Python API

      .. important:: 
         
            Ensure the QRMI Python package is installed in your Python virtual
            environment (e.g. ``~/py312_qrmi_venv``).

      To build the Python API docs, run the following command:

      .. code:: shell-session

         source ~/py312_qrmi_venv/bin/activate
         python -m pydoc -p 8290
         Server ready at http://localhost:8290/
         Server commands: [b]rowser, [q]uit
         server> b

      The docs will be available at the following address in your browser:

      .. code:: shell-session

         http://localhost:8290/qrmi.html

      To quit the server:

      .. code:: shell-session

         server> q

   .. tab:: Rust API

      To build the Rust API docs locally, run the following command:
      
      .. code:: shell-session

         . ~/.cargo/env
         cargo doc --no-deps --open

   .. tab:: C API

      To build the C API docs locally, run the following command:

      .. code:: shell-session

         doxygen Doxyfile

      The HTML document will be created in the ``./html`` directory. Open
      ``html/index.html`` in your web browser.
