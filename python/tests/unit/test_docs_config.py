"""Regression tests for Sphinx source links without requiring Sphinx itself."""

import importlib.util
import inspect
from pathlib import Path
import sys

import pytest


@pytest.fixture(name="docs_config")
def fixture_docs_config(monkeypatch):
    """Load the repository configuration as an inspectable module."""
    path = Path(__file__).resolve().parents[3] / "docs/conf.py"
    spec = importlib.util.spec_from_file_location("qrmi_docs_config", path)
    module = importlib.util.module_from_spec(spec)
    monkeypatch.setitem(sys.modules, spec.name, module)
    spec.loader.exec_module(module)
    return module


def test_source_link_uses_fork_and_line_numbers(docs_config):
    """Objects in the checkout link to the repository with their full line span."""
    source, start = inspect.getsourcelines(docs_config.linkcode_resolve)
    link = docs_config.linkcode_resolve(
        "py", {"module": docs_config.__name__, "fullname": "linkcode_resolve"}
    )
    assert link == (
        "https://github.com/qiskit-community/qrmi/blob/main/docs/conf.py"
        f"#L{start}-L{start + len(source) - 1}"
    )


def test_external_source_has_no_checkout_link(docs_config, tmp_path, monkeypatch):
    """An installed object outside the checkout must not abort a docs build."""
    monkeypatch.setattr(docs_config, "REPO_ROOT", tmp_path)
    assert (
        docs_config.linkcode_resolve(
            "py", {"module": docs_config.__name__, "fullname": "linkcode_resolve"}
        )
        is None
    )


@pytest.mark.parametrize(
    "domain, info",
    [
        ("rust", {"module": "qrmi_docs_config", "fullname": "linkcode_resolve"}),
        ("py", {}),
        ("py", {"module": "qrmi_docs_config"}),
        ("py", {"module": "qrmi_docs_config", "fullname": "missing"}),
        ("py", {"module": "qrmi_docs_config", "fullname": "version"}),
    ],
)
def test_objects_without_source_are_skipped(docs_config, domain, info):
    """Incomplete references and objects without Python source have no link."""
    assert docs_config.linkcode_resolve(domain, info) is None
