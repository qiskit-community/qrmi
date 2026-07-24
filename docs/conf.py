"""Sphinx configuration for the QRMI documentation."""

from pathlib import Path
import importlib
from importlib.metadata import version
import inspect

# =============================================================================
# Paths
# =============================================================================

REPO_ROOT = Path(__file__).resolve().parent.parent


# =============================================================================
# Project information
# =============================================================================

project = "Quantum Resource Management Interface (QRMI)"

author = (
    "Bacher, Utz and Birmingham, Mark and Carothers, Christopher D and "
    "Damin, Andrew and Calaza, Carlos D Gonzalez and Karnad, Ashwin Kumar "
    "and Mensa, Stefano and Moreau, Matthieu and Nober, Aurelien and "
    "Ohtani, Munetaka and others"
)

copyright = f"2026, {author}"

try:
    release = version("qrmi")
except Exception:
    release = "dev"

version = release


# =============================================================================
# General configuration
# =============================================================================

extensions = [
    "breathe",
    "sphinx.ext.autodoc",
    "sphinx.ext.linkcode",
    "sphinx.ext.napoleon",
    "sphinx_contributors",
    "sphinx_copybutton",
    "sphinx_design",
    "sphinx_tabs.tabs",
    "sphinx_togglebutton",
]

templates_path = ["_templates"]

exclude_patterns = [
    "_build/html",
    "Thumbs.db",
    ".DS_Store",
]

suppress_warnings = [
    "ref.python",
    "misc.highlighting_failure",
]

autodoc_mock_imports = [
    "iqm",
    "pulser",
    "qiskit_ibm_runtime",
    "qiskit_pasqal_provider",
]


# =============================================================================
# Breathe / Doxygen
# =============================================================================

breathe_projects = {
    "qrmi": str(REPO_ROOT / "build" / "doxygen" / "xml"),
}

breathe_default_project = "qrmi"


# =============================================================================
# Link checking
# =============================================================================

linkcheck_ignore = [
    r"../rust/qrmi/index.html",
    r"https://crates.io/crates/log",
    r"https://github.com/Qiskit/ibm-quantum-schemas/.*",
    r"https://resonance.iqm.tech/",
    # r"https://qisk.it/.*",
    # r"https://github.com/.*/tree/.*",
    # r"https://github.com/.*/blob/.*",
]

# Optional but recommended
linkcheck_timeout = 10
linkcheck_retries = 2
linkcheck_workers = 5


# =============================================================================
# Source code links
# =============================================================================

GITHUB_REPO = "https://github.com/qiskit-community/qrmi"


def linkcode_resolve(domain, info):
    """Generate GitHub source links for documented Python objects."""

    if domain != "py":
        return None

    module_name = info.get("module")
    fullname = info.get("fullname")

    if not module_name:
        return None

    try:
        module = importlib.import_module(module_name)
    except ImportError:
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
    except (TypeError, OSError):
        return None

    if filename is None:
        return None

    rel_path = Path(filename).resolve().relative_to(REPO_ROOT)
    end_lineno = lineno + len(source) - 1

    return f"{GITHUB_REPO}/blob/main/" f"{rel_path}#L{lineno}-L{end_lineno}"


# =============================================================================
# HTML output
# =============================================================================

html_theme = "shibuya"

html_static_path = ["_static"]

html_css_files = [
    "custom.css",
]

html_js_files = [
    "contributors.js",
]

html_theme_options = {
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
    "source_version": "main",
    "source_docs_path": "/docs/",
}
