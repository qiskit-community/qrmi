# -*- coding: utf-8 -*-

# This code is part of Qiskit.
#
# (C) Copyright IBM 2025
#
# This code is licensed under the Apache License, Version 2.0. You may
# obtain a copy of this license in the LICENSE.txt file in the root directory
# of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
#
# Any modifications or derivative works of this code must retain this
# copyright notice, and modified files need to carry a notice indicating
# that they have been altered from the originals.

"""qrmi_task_runner - Command to run a QRMI task"""
import os
import sys
import time
import json
import argparse
import signal
import atexit
from typing import List
import logging
from logging import getLogger, ERROR, INFO, DEBUG
from dotenv import load_dotenv
from qrmi import QuantumResource, ResourceType, Payload, TaskStatus

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
    stream=sys.stdout, level=_get_loglevel(), format="%(asctime)s %(levelname)s %(message)s"
)
logger = getLogger(__name__)


class App:
    """Application to run a QRMI task"""

    POLLING_INTERVAL_SECONDS = 1

    RESOURCE_TYPE_MAP = {
        "direct-access": ResourceType.IBMDirectAccess,
        "qiskit-runtime-service": ResourceType.IBMQiskitRuntimeService,
        "pasqal-cloud": ResourceType.PasqalCloud,
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
        self._qrmi = None

        self._name = name
        self._input_filename = input_filename
        self._output_filename = output_filename

        # setup signal handler for slurm, and start it
        signal.signal(signal.SIGTERM, self._signal_handler)
        signal.signal(signal.SIGCONT, self._signal_handler)

        atexit.register(self._exit_callback)

    def _signal_handler(self, _signal_number, _frame):
        """A signal handler to cancel this task"""
        self._is_running = False

    def _find_qpu_type(self, qpu_name: str) -> ResourceType:
        """Finds QPU type for the specified resource

        Args:
            qpu_name(str): Name of QPU for which ResourceType is desired

        Returns:
            ResourceType: ResourceType of `qpu_name`
        """
        qpu_resources = self._get_list_envvars("SLURM_JOB_QPU_RESOURCES")
        qpu_types = self._get_list_envvars("SLURM_JOB_QPU_TYPES")
        for index, qpu_resource in enumerate(qpu_resources):
            if qpu_resource == qpu_name:
                return self.RESOURCE_TYPE_MAP[qpu_types[index]]
        return ValueError(f"{qpu_name} is not available")

    def _exit_callback(self):
        """A callback called when program is finished.
        Outputs the task result if suceeded and close QRMI task
        """
        if self._qrmi is None:
            return

        if self._succeeded and self._task_id is not None:
            # write output if task was succeeded
            result = self._qrmi.task_result(self._task_id).value
            if self._output_filename:
                with open(self._output_filename, "w", encoding="utf-8") as output_file:
                    output_file.write(result)
            else:
                print(result)

        # cleanup quantum task
        if self._task_id is not None:
            self._qrmi.task_stop(self._task_id)

    @staticmethod
    def _get_list_envvars(envvar_name: str) -> List[str]:
        values = os.environ.get(envvar_name)
        if values is None:
            raise RuntimeError(
                f"The environment variable `{envvar_name}` is not set "
                "and as such configuration could not be loaded."
            )
        return values.split(",")

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
            if os.access(self._output_filename, os.W_OK) is False:
                raise RuntimeError(f"{self._output_filename} cannot be created.")

        res_type = self._find_qpu_type(self._name)
        self._qrmi = QuantumResource(self._name, res_type)

        with open(self._input_filename, encoding="utf-8") as f:
            task_input = json.load(f)
            if res_type in [
                ResourceType.IBMDirectAccess,
                ResourceType.IBMQiskitRuntimeService,
            ]:
                payload = Payload.QiskitPrimitive(
                    input=json.dumps(task_input["parameters"]),
                    program_id=task_input["program_id"],
                )
            else:
                payload = Payload.PasqalCloud(
                    sequence=json.dumps(task_input["sequence"]),
                    job_runs=task_input["job_runs"],
                )

            # start a task
            self._task_id = self._qrmi.task_start(payload)
            logger.info("Task ID: %s", self._task_id)

            # Poll the task status until it progresses to a final state such as
            # TaskStatus::Completed.
            while self._is_running:
                try:
                    status = self._qrmi.task_status(self._task_id)
                    if status == TaskStatus.Completed:
                        self._succeeded = True
                        break
                    if status in [TaskStatus.Failed, TaskStatus.Cancelled]:
                        logger.error(status)
                        break
                except Exception as err:  # pylint: disable=broad-except
                    logger.error(
                        "Failed to get task status. reason = %s. Retrying.", err
                    )
                time.sleep(self.POLLING_INTERVAL_SECONDS)

            self._is_running = False


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
