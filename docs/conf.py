# Configuration file for the Sphinx documentation builder.
#
# For the full list of built-in configuration values, see the documentation:
# https://www.sphinx-doc.org/en/master/usage/configuration.html

# -- Project information -----------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#project-information

project = "Quantum Resource Management Interface (QRMI)"
copyright = "2026, Bacher, Utz and Birmingham, Mark and Carothers, Christopher D and Damin, Andrew and Calaza, Carlos D Gonzalez and Karnad, Ashwin Kumar and Mensa, Stefano and Moreau, Matthieu and Nober, Aurelien and Ohtani, Munetaka and others"
author = "Bacher, Utz and Birmingham, Mark and Carothers, Christopher D and Damin, Andrew and Calaza, Carlos D Gonzalez and Karnad, Ashwin Kumar and Mensa, Stefano and Moreau, Matthieu and Nober, Aurelien and Ohtani, Munetaka and others"
release = "2025"

# -- General configuration ---------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#general-configuration

extensions = []

templates_path = ["_templates"]
exclude_patterns = ["_build", "Thumbs.db", ".DS_Store"]

# -- Options for HTML output -------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#options-for-html-output

html_theme = "alabaster"
html_static_path = ["_static"]
