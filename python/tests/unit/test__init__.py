"""Unit tests for the `__init__.py` module."""

from unittest.mock import Mock

import pytest
import qrmi
from qrmi import get_job_qpu_resources_and_types


def test_get_job_qpu_resources_and_types_len_error(monkeypatch):
    """Test ValueError is raised when resources and types lengths differ."""
    mock = Mock(
        side_effect=[
            ["qpu1", "qpu2"],
            ["pasqal-cloud"],
        ]
    )

    monkeypatch.setattr(
        qrmi,
        "_get_job_env_list",
        mock,
    )

    with pytest.raises(ValueError):
        get_job_qpu_resources_and_types()


def test_get_job_qpu_resources_and_types_logs_warning(monkeypatch, caplog):
    """Test warning is logged when no QPU resources are configured."""

    monkeypatch.setattr(
        qrmi,
        "_get_job_env_list",
        lambda *args: [],
    )

    qpus, qpu_types = get_job_qpu_resources_and_types()

    assert qpus == []
    assert qpu_types == []
    assert "No QPU resources or types specified." in caplog.text
