from unittest.mock import patch, MagicMock

import pytest

from qiskit.providers import JobStatus
from qrmi import TaskStatus
from qrmi.primitives.runtime_job_v2 import RuntimeJobV2


class _FakeResult:
    def __init__(self, value):
        self.value = value


class _FakeQRMI:
    def __init__(self):
        self.stopped_job = None
        self.logs_value = "test logs"
        self.status_calls = 0

    def task_stop(self, job_id):
        self.stopped_job = job_id

    def task_status(self, job_id):
        self.status_calls += 1
        return TaskStatus.Completed

    def task_logs(self, job_id):
        return self.logs_value

    def task_result(self, job_id):
        return _FakeResult(value="test result")


def test_constructor_stores_job_id():
    """Verify the constructor stores the supplied job identifier."""

    job = RuntimeJobV2(_FakeQRMI(), "job-123")

    assert job.job_id() == "job-123"


def test_cancel_stops_job():
    """Verify cancelling a job stops the corresponding QRMI task."""

    qrmi = _FakeQRMI()
    job = RuntimeJobV2(qrmi, "job-123")

    job.cancel()

    assert qrmi.stopped_job == "job-123"


def test_destructor_stops_job_when_enabled():
    """Verify the QRMI task is stopped when delete_job is enabled."""

    qrmi = _FakeQRMI()
    job = RuntimeJobV2(qrmi, "job-123", delete_job=True)

    job.__del__()

    assert qrmi.stopped_job == "job-123"


def test_destructor_does_not_stop_job_by_default():
    """Verify the QRMI task is not stopped when delete_job is disabled."""

    qrmi = _FakeQRMI()
    job = RuntimeJobV2(qrmi, "job-123", delete_job=False)

    job.__del__()

    assert qrmi.stopped_job is None


@pytest.mark.parametrize(
    ("task_status", "expected"),
    [
        (TaskStatus.Queued, JobStatus.QUEUED),
        (TaskStatus.Running, JobStatus.RUNNING),
        (TaskStatus.Completed, JobStatus.DONE),
        (TaskStatus.Failed, JobStatus.ERROR),
        (TaskStatus.Cancelled, JobStatus.CANCELLED),
    ],
)
def test_status_mapping(task_status, expected):
    """Map QRMI task states to Qiskit job states."""

    qrmi = _FakeQRMI()
    job = RuntimeJobV2(qrmi, "job-123")

    # Patch the QRMI task_status method to return the desired task status
    qrmi.task_status = lambda job_id: task_status

    assert job.status() == expected


def test_final_status_caching():
    """Verify that final job states are cached."""

    qrmi = _FakeQRMI()
    qrmi.task_status = MagicMock(return_value=TaskStatus.Completed)

    job = RuntimeJobV2(qrmi, "job-123")

    assert job.status() == JobStatus.DONE
    assert job.status() == JobStatus.DONE
    assert qrmi.task_status.call_count == 2


def test_done_returns_true_for_completed_job():
    """Verify that completed jobs are reported as done."""

    qrmi = _FakeQRMI()
    job = RuntimeJobV2(qrmi, "job-123")

    # Patch the QRMI task_status method to return Completed
    qrmi.task_status = lambda job_id: TaskStatus.Completed

    assert job.done() is True


def test_running_returns_true_for_running_job():
    """Verify that running jobs are reported as running."""

    qrmi = _FakeQRMI()
    job = RuntimeJobV2(qrmi, "job-123")

    # Patch the QRMI task_status method to return Running
    qrmi.task_status = lambda job_id: TaskStatus.Running

    assert job.running() is True


def test_cancelled_returns_true_for_cancelled_job():
    """Verify that cancelled jobs are reported correctly."""

    qrmi = _FakeQRMI()
    job = RuntimeJobV2(qrmi, "job-123")

    # Patch the QRMI task_status method to return Cancelled
    qrmi.task_status = lambda job_id: TaskStatus.Cancelled

    assert job.cancelled() is True


def test_errored_returns_true_for_failed_job():
    """Verify that failed jobs are reported correctly."""

    qrmi = _FakeQRMI()
    job = RuntimeJobV2(qrmi, "job-123")

    # Patch the QRMI task_status method to return Failed
    qrmi.task_status = lambda job_id: TaskStatus.Failed

    assert job.errored() is True


def test_in_final_state_returns_true_for_completed_job():
    """Verify that completed jobs are recognised as being in a final state."""

    qrmi = _FakeQRMI()
    job = RuntimeJobV2(qrmi, "job-123")

    # Patch the QRMI task_status method to return Completed
    qrmi.task_status = lambda job_id: TaskStatus.Completed

    assert job.in_final_state() is True


def test_in_final_state_returns_false_for_running_job():
    """Verify that running jobs are recognised as not being in a final state."""

    qrmi = _FakeQRMI()
    job = RuntimeJobV2(qrmi, "job-123")

    # Patch the QRMI task_status method to return Running
    qrmi.task_status = lambda job_id: TaskStatus.Running

    assert job.in_final_state() is False


def test_logs_returns_qrmi_logs():
    """Verify that logs are returned from the QRMI backend."""

    qrmi = _FakeQRMI()

    job = RuntimeJobV2(qrmi, "job-123")

    assert job.logs() == "test logs"


def test_result_is_cached():
    """Verify that cached results are returned after the first retrieval."""

    qrmi = _FakeQRMI()
    job = RuntimeJobV2(qrmi, "job-123")

    # Patch the QRMI task_result method to return a fake result
    with patch(
        "qrmi.primitives.runtime_job_v2.ResultDecoder.decode",
        return_value="decoded result",
    ):
        first = job.result()
        second = job.result()

    assert first == "decoded result"
    assert second == "decoded result"
    assert qrmi.status_calls == 1


def test_result_waits_for_completion():
    """Verify that result() waits for the job to complete before returning."""

    qrmi = _FakeQRMI()
    job = RuntimeJobV2(qrmi, "job-123")

    # Patch the QRMI task_status method to simulate a running job followed by completion
    status_sequence = [TaskStatus.Running, TaskStatus.Running, TaskStatus.Completed]
    qrmi.task_status = lambda job_id: (
        status_sequence.pop(0) if status_sequence else TaskStatus.Completed
    )

    # Patch the QRMI task_result method to return a fake result
    qrmi.task_result = lambda job_id: _FakeResult(value="fake result")

    with patch("qrmi.primitives.runtime_job_v2.time.sleep", return_value=None):
        result = job.result()

    assert result == "fake result"


def test_result_decodes_payload():
    """Verify that the result payload is decoded correctly using the IBM result decoder."""

    qrmi = _FakeQRMI()
    job = RuntimeJobV2(qrmi, "job-123")

    # Patch the QRMI task_result method to return a fake encoded result
    qrmi.task_result = lambda job_id: _FakeResult(value="encoded result")

    # Patch the ResultDecoder.decode method to simulate decoding
    with patch(
        "qrmi.primitives.runtime_job_v2.ResultDecoder.decode",
        return_value="decoded result",
    ) as decode:
        result = job.result()

    decode.assert_called_once_with("encoded result")
    assert result == "decoded result"


def test_result_polls_until_final_state():
    """Verify that result() polls the job status until it reaches a final state."""

    qrmi = _FakeQRMI()
    job = RuntimeJobV2(qrmi, "job-123")

    # Patch the QRMI task_status method to simulate a running job followed by completion
    status_sequence = [TaskStatus.Running, TaskStatus.Running, TaskStatus.Completed]
    qrmi.task_status = lambda job_id: (
        status_sequence.pop(0) if status_sequence else TaskStatus.Completed
    )

    # Patch the QRMI task_result method to return a fake result
    qrmi.task_result = lambda job_id: _FakeResult(value="fake result")

    with patch("qrmi.primitives.runtime_job_v2.time.sleep", return_value=None):
        result = job.result()

    assert result == "fake result"
