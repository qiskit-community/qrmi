# -*- coding: utf-8 -*-

# This code is part of Qiskit.
#
# (C) Copyright 2025, 2026 IBM. 2026 Pasqal. All Rights Reserved.
#
# This code is licensed under the Apache License, Version 2.0. You may
# obtain a copy of this license in the LICENSE.txt file in the root directory
# of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
#
# Any modifications or derivative works of this code must retain this
# copyright notice, and modified files need to carry a notice indicating
# that they have been altered from the originals.

"""QRMI python package"""

import os
from logging import getLogger

from ._core import *  # noqa: F403  # pylint: disable=import-error

logger = getLogger("qrmi")


def _get_job_env_list(envvar_name: str, legacy_envvar_name: str | None) -> list[str]:
    """Return a QRMI job environment list.

    Args:
        envvar_name (str): QRMI environment variable name.
        legacy_envvar_name (str): Legacy environment variable name.

    Returns:
        (list[str]): Environment variable values split with the QRMI delimiter.
    """
    values = os.environ.get(envvar_name)
    if values is None and legacy_envvar_name is not None:
        values = os.environ.get(legacy_envvar_name)
    if values is None:
        raise RuntimeError(
            f"The environment variable `{envvar_name}` is not set and as such configuration "
            "could not be loaded."
        )
    sep = os.environ.get(key="QRMI_LIST_DELIMITER", default=",")
    return [] if len(values) == 0 else values.split(sep)


def get_job_qpu_resources_and_types() -> tuple[list[str], list[str]]:
    """Return QRMI job QPU resources and types.

    Returns:
        Tuple[list[str], list[str]]: A tuple containing the list of QPU resources and their corresponding types.
    """
    qpus = _get_job_env_list("QRMI_JOB_QPU_RESOURCES", "SLURM_JOB_QPU_RESOURCES")
    qpu_types = _get_job_env_list("QRMI_JOB_QPU_TYPES", "SLURM_JOB_QPU_TYPES")
    if len(qpus) != len(qpu_types):
        raise ValueError(
            f"Inconsistent specifications of QPU resources and types. {qpus} vs {qpu_types}"
        )
    if len(qpus) == len(qpu_types) == 0:
        logger.warning("No QPU resources or types specified.")
    return qpus, qpu_types
