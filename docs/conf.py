# Configuration file for the Sphinx documentation builder.
#
# For the full list of built-in configuration values, see the documentation:
# https://www.sphinx-doc.org/en/master/usage/configuration.html

import inspect
import os
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent

# -- Project information -----------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#project-information

project = "Quantum Resource Management Interface (QRMI)"
copyright = "2026, Bacher, Utz and Birmingham, Mark and Carothers, Christopher D and Damin, Andrew and Calaza, Carlos D Gonzalez and Karnad, Ashwin Kumar and Mensa, Stefano and Moreau, Matthieu and Nober, Aurelien and Ohtani, Munetaka and others"
author = "Bacher, Utz and Birmingham, Mark and Carothers, Christopher D and Damin, Andrew and Calaza, Carlos D Gonzalez and Karnad, Ashwin Kumar and Mensa, Stefano and Moreau, Matthieu and Nober, Aurelien and Ohtani, Munetaka and others"
release = "2025"

# version = qrmi.__version__


# -- General configuration ---------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#general-configuration

extensions = [
    "sphinx_design",
    "sphinx_tabs.tabs",
    "sphinx_contributors",
    "sphinx.ext.autodoc",
    "sphinx.ext.napoleon",
    "sphinx.ext.linkcode",
    "sphinx_copybutton",
    "sphinx_togglebutton",
    "breathe",
]

breathe_projects = {
    "qrmi": f"{REPO_ROOT}/build/doxygen/xml",
}

breathe_default_project = "qrmi"


# -- linkcode configuration --------------------------------------------------


def linkcode_resolve(domain, info):
    """
    Resolve a GitHub source URL for a documented Python object.

    :param domain: Sphinx domain of the object.
    :param info: Metadata describing the documented object.
    :returns: GitHub URL for the object's source code, or ``None``.
    """

    if domain != "py":
        return None

    module_name = info["module"]
    fullname = info["fullname"]

    if not module_name:
        return None

    try:
        module = sys.modules[module_name]
    except KeyError:
        return None

    obj = module

    for part in fullname.split("."):
        try:
            obj = getattr(obj, part)
        except AttributeError:
            return None

    try:
        filename = inspect.getsourcefile(obj)
        source, lineno = inspect.getsourcelines(obj)
    except Exception:
        return None

    rel_path = Path(filename).resolve().relative_to(REPO_ROOT)

    end_lineno = lineno + len(source) - 1

    branch = "main"

    url = (
        f"https://github.com/qiskit-community/qrmi/blob/{branch}/"
        f"{rel_path}#L{lineno}-L{end_lineno}"
    )

    return url


# # Mock optional dependencies for sphinx-apidocs
autodoc_mock_imports = [
    "iqm",
    "qiskit_ibm_runtime",
    "qiskit_pasqal_provider",
    "pulser",
]


templates_path = ["_templates"]
exclude_patterns = ["_build/html", "Thumbs.db", ".DS_Store"]


# -- Options for HTML output -------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#options-for-html-output

html_theme = "shibuya"

html_static_path = ["_static"]

html_css_files = [
    "custom.css",
]

html_js_files = [
    "contributors.js",
]

html_theme_options = {
    # "light_logo": "logo-light.svg",
    # "dark_logo": "logo-dark.svg",
    # "og_image_url": "https://example.com/icon.png",
    "discussion_url": "https://github.com/qiskit-community",
    "github_url": "https://github.com/qiskit-community/qrmi",
    "nav_links": [
        {
            "title": "SPANK Plugins",
            "url": "https://github.com/qiskit-community/spank-plugins",
            "external": True,
        },
    ],
}

html_context = {
    "source_type": "github",
    "source_user": "qiskit-community",
    "source_repo": "qrmi",
    "source_version": "main",  # Optional
    "source_docs_path": "/docs/",  # Optional
}
