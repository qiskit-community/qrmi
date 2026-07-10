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

Theme
~~~~~

This documentation uses the `Shibuya`_ theme. 

.. _Shibuya: https://shibuya.lepture.com/

Adding Documentation
--------------------

All documentation files are stored in the `docs` directory. The `index.rst` file defines the content of the landing page, as well as structure of the documentation (as seen in the sidebar).

If you would like to add to the existing documentation, follow these steps:

#. Create a new reStructured Text (`.rst`) file in the `docs` directory. If the file relates to an existing topic, you can place it in the appropriate subdirectory.

#. In `docs/index.rst`, add a reference to the new file in the appropriate section of the `toctree` directive. For example, for a new file called `new_topic.rst`:

   .. code-block:: rst

      .. toctree::
         :maxdepth: 2
         :caption: New Section

         new_topic

#. 