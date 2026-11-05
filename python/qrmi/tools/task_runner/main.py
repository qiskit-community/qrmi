# This code is part of Qiskit.
#
# (C) Copyright IBM 2025, 2026
# (C) Copyright UKRI-STFC 2026
#
# This code is licensed under the Apache License, Version 2.0. You may
# obtain a copy of this license in the LICENSE.txt file in the root directory
# of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
#
# Any modifications or derivative works of this code must retain this
# copyright notice, and modified files need to carry a notice indicating
# that they have been altered from the originals.

"""qrmi_task_runner - Command to run a QRMI task"""

import argparse
import json
import logging
import os
import signal
import sys
import time
from logging import DEBUG, ERROR, INFO, getLogger
from pathlib import Path

from dotenv import load_dotenv
from qrmi import (
    Payload,
    QuantumResource,
    ResourceType,
    TaskStatus,
    get_job_qpu_resources_and_types,
)

load_dotenv()


def _get_loglevel() -> int:
    """Converts SRUN_DEBUG to python logging level and set it. Default is INFO."""
    srun_debug = os.environ.get("SRUN_DEBUG")
    if srun_debug is None:
        return INFO

    level = INFO
    try:
        level_ivalue = int(srun_debug)
    except ValueError:
        # default is Info as same as srun
        return level

    if level_ivalue == 2:
        # --quiet
        level = ERROR
    elif level_ivalue >= 4:
        # --verbose
        # -vv or more
        level = DEBUG

    return level


logging.basicConfig(
    stream=sys.stdout,
    level=_get_loglevel(),
    format="%(asctime)s %(levelname)s %(message)s",
)
logger = getLogger(__name__)


class App:
    """Application to run a QRMI task"""

    POLLING_INTERVAL_SECONDS = 1

    RESOURCE_TYPE_MAP = {
        "ibm-quantum-system": ResourceType.IBMQuantumSystem,
        "ibm-quantum-compute-service": ResourceType.IBMQuantumComputeService,
        "qiskit-runtime-service": ResourceType.IBMQiskitRuntimeService,
        "pasqal-cloud": ResourceType.PasqalCloud,
        "iqm-server": ResourceType.IQMServer,
        "alice-bob-felis": ResourceType.AliceBobFelis,
        "oqtopus": ResourceType.OQTOPUS,
    }

    def __init__(self, name: str, input_filename: str, output_filename: str):
        """Constructs an application.

        Args:
            name(str): QPU resource name
            input_filename(str): Input filename
            output_filename(str): Name of the file to save the results to.
                                  Use stdout if None is specified.
        """
        self._is_running = True
        self._task_id = None
        self._succeeded = False
        self._task_terminal = False
        self._qrmi = None
        self._finalized = False

        self._name = name
        self._input_filename = input_filename
        self._output_filename = output_filename

        # setup signal handler for slurm, and start it
        signal.signal(signal.SIGTERM, self._signal_handler)
        signal.signal(signal.SIGCONT, self._signal_handler)

    def _signal_handler(self, signal_number, _frame):
        """A signal handler to cancel this task.

        This must only ever flip `_is_running` and return -- it must
        NOT call into `self._qrmi` (e.g. via `_finalize()`) directly,
        even though SIGTERM/SIGCONT handlers run during ordinary
        execution (unlike `atexit`, see `_finalize()`'s docstring).

        The reason is specific to how the QRMI bindings for backends
        such as OQTOPUS implement `&mut self` methods like
        `task_status()`/`task_stop()`/`task_result()`: each one
        releases the GIL (`py.detach(...)`) to block on a Tokio runtime,
        and then *reacquires* the GIL from inside that block to call
        back into Python (`oqtopus_client`'s `asyncio` event loop). That
        reacquired, nested Python execution is a point where a pending
        signal legitimately CAN be delivered and this handler invoked
        -- while the *outer* `self._qrmi` call (e.g. `task_status()`)
        is still on the stack, still holding PyO3's runtime-checked
        exclusive (`&mut self`) borrow on that object. Calling another
        `self._qrmi` method (e.g. `task_stop()`) from here, in that
        window, hits that same borrow and fails immediately with
        `RuntimeError: Already borrowed` -- confirmed by reproducing
        this exact pattern in isolation. It isn't a crash (PyO3's
        runtime borrow check catches it cleanly), but the call never
        happens, silently defeating the whole point of handling the
        signal.

        `_is_running = False` here is just a plain attribute write, so
        it's always safe regardless of what `self._qrmi` is doing. The
        polling loop in `run()` picks it up on its next iteration --
        by construction, at that point no other call into `self._qrmi`
        is in flight (the previous one, if any, has already fully
        returned) -- and `_finalize()` runs from there instead, in its
        `finally` block, where it's actually safe to touch `self._qrmi`.

        Note this can't help with SIGKILL (or SIGSTOP) regardless:
        those can't be intercepted by any process, in any language --
        the kernel terminates the process before any handler, `atexit`
        callback, or `finally` block gets a chance to run. If jobs need
        to be cleaned up after a hard kill (e.g. Slurm's wall-time
        SIGKILL), that has to happen from outside this process (e.g. an
        earlier `sbatch --signal=TERM@<seconds>` warning, or an
        external reaper/epilog script that cancels orphaned jobs by
        ID).
        """
        logger.info("received signal %s, stopping task", signal_number)
        self._is_running = False

    def _find_qpu_type(self, qpu_name: str) -> ResourceType:
        """Finds QPU type for the specified resource

        Args:
            qpu_name(str): Name of QPU for which ResourceType is desired

        Returns:
            ResourceType: ResourceType of `qpu_name`
        """
        qpu_resources, qpu_types = get_job_qpu_resources_and_types()
        for index, qpu_resource in enumerate(qpu_resources):
            if qpu_resource == qpu_name:
                return self.RESOURCE_TYPE_MAP[qpu_types[index]]
        raise ValueError(f"{qpu_name} is not available")

    def _finalize(self):
        """Writes the task result (if succeeded) and stops/cleans up the
        quantum task. Safe to call more than once; only does the work
        once.

        Only ever called from `run()`'s own `finally` block (for the
        normal/successful completion path, and as a catch-all if
        anything in `run()` raised, including the loop exiting early
        because `_signal_handler` set `_is_running = False`) -- never
        from `_signal_handler` itself, and never deferred to `atexit`.
        Both of those would be unsafe, for different reasons:

        - `atexit`: backends such as OQTOPUS call into `oqtopus_client`,
          whose synchronous wrapper methods spin up a fresh `asyncio`
          event loop per call, and `aiohttp` occasionally offloads part
          of the request (e.g. proxy/auth resolution) to the default
          `ThreadPoolExecutor` via `loop.run_in_executor(...)`. CPython
          always runs `threading._shutdown()` -- which tears down that
          default executor -- *before* it runs plain
          `atexit`-registered callbacks, so any such network call made
          from an `atexit` callback is liable to fail with
          `RuntimeError: cannot schedule new futures after interpreter
          shutdown`, even though the task itself already succeeded.

        - directly from `_signal_handler`: QRMI's `&mut self` bindings
          for `task_status()`/`task_stop()`/`task_result()` release the
          GIL to block on a Tokio runtime and then reacquire it to call
          back into Python from inside that same call. A signal can be
          delivered (and this handler invoked) during that reacquired,
          nested Python execution -- i.e. while an outer call such as
          `task_status()` is still on the stack, still holding PyO3's
          exclusive borrow on `self._qrmi`. Calling another method on
          the same object from there (e.g. `task_stop()`) hits that
          same borrow and fails immediately with `RuntimeError: Already
          borrowed` (confirmed by reproducing this exact pattern in
          isolation). Calling `_finalize()` only from `run()`'s
          `finally` block sidesteps this entirely: by the time control
          reaches it, any previous `self._qrmi` call has already fully
          returned, so there's nothing left to conflict with.
        """
        if self._finalized or self._qrmi is None:
            return
        self._finalized = True

        if self._succeeded and self._task_id is not None:
            try:
                # write output if task was succeeded
                result = self._qrmi.task_result(self._task_id).value
                if self._output_filename:
                    with open(
                        self._output_filename, "w", encoding="utf-8"
                    ) as output_file:
                        output_file.write(result)
                else:
                    print(result)
            except Exception as err:  # pylint: disable=broad-except
                logger.error("Failed to fetch task result. reason = %s", err)

        # cleanup quantum task -- only actually cancel it if it's still
        # running/queued. If it already reached a terminal state
        # (Completed/Failed/Cancelled; `_task_terminal` is set in that
        # case), calling task_stop() on it is pointless and some
        # backends reject cancelling an already-finished job as an
        # error, which would just add noise to the log for no reason.
        if self._task_id is not None and not self._task_terminal:
            try:
                self._qrmi.task_stop(self._task_id)
            except Exception as err:  # pylint: disable=broad-except
                logger.error("Failed to stop task. reason = %s", err)

    @property
    def is_running(self) -> bool:
        """Return True if QRMI task is running"""
        return self._is_running

    @property
    def task_id(self) -> str:
        """Return a task identifier if available, otherwise None"""
        return self._task_id

    def run(self) -> None:
        """app main()"""
        # Before executing a quantum job, check to see if the specified
        # file can be created, and inform to user if it cannot be written. This is
        # to prevent file writing errors after a long job execution.
        if self._output_filename:
            # verify that the destination directory is writable
            dir_path = Path(self._output_filename).parent
            if os.access(dir_path, os.W_OK) is False:
                raise RuntimeError(f"{self._output_filename} cannot be created.")

        res_type = self._find_qpu_type(self._name)
        self._qrmi = QuantumResource(self._name, res_type)

        # Wrapped in try/finally (ordinary control flow -- the
        # interpreter is still fully alive here, unlike `atexit`) so
        # `_finalize()` always runs exactly once no matter how this
        # block exits: the task completed normally, a signal handler
        # already finalized and this is just unwinding, or something
        # here raised an exception. `_finalize()` itself is idempotent,
        # so whichever path gets there first (this `finally`, or
        # `_signal_handler`) does the real work and the other is a
        # no-op.
        try:
            with open(self._input_filename, encoding="utf-8") as input_file:
                task_input = json.load(input_file)
                if res_type in [
                    ResourceType.IBMQuantumSystem,
                    ResourceType.IBMQuantumComputeService,
                    ResourceType.IBMQiskitRuntimeService,
                ]:
                    payload = Payload.QiskitPrimitive(
                        input=json.dumps(task_input["parameters"]),
                        program_id=task_input["program_id"],
                    )
                elif res_type in [
                    ResourceType.IQMServer,
                ]:
                    payload = Payload.IQMServer(
                        iqmjson=json.dumps(task_input["iqmjson"]),
                        use_timeslot=task_input["use_timeslot"],
                        tag=task_input["tag"],
                        job_type=task_input["job_type"],
                    )
                elif res_type in [
                    ResourceType.AliceBobFelis,
                ]:
                    payload = Payload.AliceBobFelis(
                        human_qir=json.dumps(task_input["human_qir"]),
                        input_params=json.dumps(task_input["input_params"]),
                    )
                elif res_type in [
                    ResourceType.OQTOPUS,
                ]:
                    payload = Payload.Oqtopus(
                        job_spec=json.dumps(task_input),
                    )
                else:
                    payload = Payload.PasqalCloud(
                        sequence=json.dumps(task_input["sequence"]),
                        job_runs=task_input["job_runs"],
                    )

                # start a task
                self._task_id = self._qrmi.task_start(payload)
                logger.info("Task ID: %s", self._task_id)

                # Poll the task status until it progresses to a final state
                # such as TaskStatus::Completed.
                while self._is_running:
                    try:
                        status = self._qrmi.task_status(self._task_id)
                        if status == TaskStatus.Completed:
                            self._succeeded = True
                            self._task_terminal = True
                            break
                        if status in [TaskStatus.Failed, TaskStatus.Cancelled]:
                            logger.error(status)
                            self._task_terminal = True
                            break
                    except Exception as err:  # pylint: disable=broad-except
                        logger.error(
                            "Failed to get task status. reason = %s. Retrying.",
                            err,
                        )
                    time.sleep(self.POLLING_INTERVAL_SECONDS)

                self._is_running = False
        finally:
            self._finalize()


def run() -> None:
    """Entrypoint to run a task"""
    parser = argparse.ArgumentParser(
        description="qrmi_task_runner - Command to run a QRMI task"
    )
    parser.add_argument("name", help="QPU resource name")
    parser.add_argument("input", help="Input file")
    parser.add_argument(
        "output", nargs="?", help="Write output to <file> instead of stdout"
    )
    args = parser.parse_args()

    App(args.name, args.input, args.output).run()


if __name__ == "__main__":
    run()
