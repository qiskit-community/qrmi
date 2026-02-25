# This code is part of Qiskit.
#
# (C) Copyright 2025, 2026 IBM. All Rights Reserved.
#
# This code is licensed under the Apache License, Version 2.0. You may
# obtain a copy of this license in the LICENSE.txt file in the root directory
# of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
#
# Any modifications or derivative works of this code must retain this
# copyright notice, and modified files need to carry a notice indicating
# that they have been altered from the originals.

"""IBMBackend implementation with IBM QRMI"""

import json
from typing import Any, List, Optional
import dateutil

from qiskit import QuantumCircuit
from qiskit.providers.backend import Backend, BackendV2
from qiskit.transpiler.target import Target
from qiskit.providers.options import Options
from qiskit.result import MeasLevel, MeasReturnType
from qiskit_ibm_runtime.utils.backend_decoder import (
    properties_from_server_data,
    configuration_from_server_data,
)
from qiskit_ibm_runtime.utils.backend_converter import convert_to_target
from qiskit_ibm_runtime.exceptions import IBMBackendError
from qiskit_ibm_runtime.models import (
    BackendProperties,
    QasmBackendConfiguration,
    BackendStatus,
)
from qrmi import QuantumResource  # pylint: disable=no-name-in-module


def get_backend(
    qrmi: QuantumResource, use_fractional_gates: Optional[bool] = False
) -> Backend:
    """Returns Qiskit transpiler target

    Args:
        qrmi: IBM QRMI object
        use_fractional_gates: Whether to use native “fractional gates” on the device if available.

    Returns:
        qiskit.transpiler.target.Target: Qiskit Transpiler target
    """
    return QRMIBackend(
        qrmi,
        use_fractional_gates=use_fractional_gates,
    )


class QRMIBackend(BackendV2):
    """Backend class interfacing with an IBM Quantum backend."""

    def __init__(
        self,
        qrmi: QuantumResource,
        **fields,
    ):
        self._qrmi = qrmi
        target = self._qrmi.target()
        target = json.loads(target.value)
        config_dict = target["configuration"]
        prop_dict = target["properties"]

        super().__init__(
            name=config_dict["backend_name"],
            online_date=dateutil.parser.isoparse(config_dict["online_date"]),
            backend_version=config_dict["backend_version"],
        )
        if fields:
            self.set_options(**fields)

        self._configuration = configuration_from_server_data(
            config_dict, use_fractional_gates=self.options.use_fractional_gates
        )
        self._properties = properties_from_server_data(
            prop_dict, use_fractional_gates=self.options.use_fractional_gates
        )
        self._target = convert_to_target(
            configuration=self._configuration,  # type: ignore[arg-type]
            properties=self._properties,
        )

    def __getattr__(self, name: str) -> Any:
        """Gets attribute from self or configuration

        This magic method executes when user accesses an attribute that
        does not yet exist on QRMIBackend class.
        """
        # Prevent recursion since these properties are accessed within __getattr__
        if name in ["_properties", "_target", "_configuration"]:
            raise AttributeError(
                f"'{self.__class__.__name__}' object has no attribute '{name}'"
            )

        # Check if the attribute now is available on QRMIBackend class due to above steps
        try:
            return super().__getattribute__(name)
        except AttributeError:
            pass
        # If attribute is still not available on QRMIBackend class,
        # fallback to check if the attribute is available in configuration
        try:
            return self._configuration.__getattribute__(name)
        except AttributeError as ex:
            raise AttributeError(
                f"'{self.__class__.__name__}' object has no attribute '{name}'"
            ) from ex

    @classmethod
    def _default_options(cls) -> Options:
        """Default runtime options."""
        return Options(
            shots=4000,
            memory=False,
            meas_level=MeasLevel.CLASSIFIED,
            meas_return=MeasReturnType.AVERAGE,
            memory_slots=None,
            memory_slot_size=100,
            rep_time=None,
            rep_delay=None,
            init_qubits=True,
            use_measure_esp=None,
            use_fractional_gates=False,
            # Simulator only
            noise_model=None,
            seed_simulator=None,
        )

    def _convert_to_target(self, refresh: bool = False) -> None:
        """Converts backend configuration and properties to Target object"""
        if refresh or not self._target:
            target = self._qrmi.target()
            target = json.loads(target.value)
            self._configuration = configuration_from_server_data(
                target["configuration"],
                use_fractional_gates=self.options.use_fractional_gates,
            )
            self._properties = properties_from_server_data(
                target["properties"],
                use_fractional_gates=self.options.use_fractional_gates,
            )
            self._target = convert_to_target(
                configuration=self._configuration,  # type: ignore[arg-type]
                prperties=self._properties,
            )

    @property
    def dtm(self) -> float:
        """Return the system time resolution of output signals

        Returns:
            dtm: The output signal timestep in seconds.
        """
        return self._configuration.dtm

    @property
    def max_circuits(self) -> None:
        """This property used to return the `max_experiments` value from the
        backend configuration but this value is no longer an accurate representation
        of backend circuit limits. New fields will be added to indicate new limits.
        """
        return None

    @property
    def meas_map(self) -> List[List[int]]:
        """Return the grouping of measurements which are multiplexed

        This is required to be implemented if the backend supports Pulse
        scheduling.

        Returns:
            meas_map: The grouping of measurements which are multiplexed
        """
        return self._configuration.meas_map

    def refresh(self) -> None:
        """Retrieve the newest backend configuration and refresh the current backend target."""
        self._convert_to_target(refresh=True)

    @property
    def target(self) -> Target:
        """A :class:`qiskit.transpiler.Target` object for the backend.

        Returns:
            Target
        """
        return self._target

    def configuration(
        self,
    ) -> QasmBackendConfiguration:
        """Return the backend configuration.

        Backend configuration contains fixed information about the backend, such
        as its name, number of qubits, basis gates, coupling map, quantum volume, etc.

        The schema for backend configuration can be found in
        `Qiskit/ibm-quantum-schemas/backend_configuration
        <https://github.com/Qiskit/ibm-quantum-schemas/blob/main/schemas/backend_configuration_schema.json>`_.

        More details about backend configuration properties can be found here
        `QasmBackendConfiguration
        <https://quantum.cloud.ibm.com/docs/api/qiskit/1.4/qiskit.providers.models.QasmBackendConfiguration>`_.

        IBM backends may also include the following properties:
            * ``supported_features``: a list of strings of supported features like "qasm3"
                for dynamic circuits support.
            * ``parallel_compilation``: a boolean of whether or not the backend can process multiple
                jobs at once. Parts of the classical computation will be parallelized.

        Returns:
            The configuration for the backend.
        """
        return self._configuration

    def properties(
        self,
        refresh: bool = False,
    ) -> Optional[BackendProperties]:
        """Return the backend properties, subject to optional filtering.

        This data describes qubits properties (such as T1 and T2),
        gates properties (such as gate length and error), and other general
        properties of the backend.

        The schema for backend properties can be found in
        `Qiskit/ibm-quantum-schemas/backend_properties
        <https://github.com/Qiskit/ibm-quantum-schemas/blob/main/schemas/backend_properties_schema.json>`_.

        Args:
            refresh: If ``True``, re-query the server for the backend properties.
                Otherwise, return a cached version.

        Returns:
            The backend properties or ``None`` if the backend properties are not
            currently available.

        Raises:
            TypeError: If an input argument is not of the correct type.
            NotImplementedError: If `datetime` is specified when cloud runtime is used.
        """
        if refresh or self._properties is None:
            target = self._qrmi.target()
            target = json.loads(target.value)
            self._properties = properties_from_server_data(
                target["properties"],
                use_fractional_gates=self.options.use_fractional_gates,
            )

        return self._properties

    def check_faulty(self, circuit: QuantumCircuit) -> None:
        """Check if the input circuit uses faulty qubits or edges.

        Args:
            circuit: Circuit to check.

        Raises:
            ValueError: If an instruction operating on a faulty qubit or edge is found.
        """
        if not self.properties():
            return

        faulty_qubits = self.properties().faulty_qubits()
        faulty_gates = self.properties().faulty_gates()
        faulty_edges = [
            tuple(gate.qubits) for gate in faulty_gates if len(gate.qubits) > 1
        ]

        for instr in circuit.data:
            if instr.operation.name == "barrier":
                continue
            qubit_indices = tuple(circuit.find_bit(x).index for x in instr.qubits)

            for circ_qubit in qubit_indices:
                if circ_qubit in faulty_qubits:
                    raise ValueError(
                        f"Circuit {circuit.name} contains instruction "
                        f"{instr} operating on a faulty qubit {circ_qubit}."
                    )

            if len(qubit_indices) == 2 and qubit_indices in faulty_edges:
                raise ValueError(
                    f"Circuit {circuit.name} contains instruction "
                    f"{instr} operating on a faulty edge {qubit_indices}"
                )

    def run(self, *args, **kwargs) -> None:  # type: ignore[no-untyped-def]
        """
        Raises:
            IBMBackendError: The run() method is no longer supported.

        """
        raise IBMBackendError(
            "Support for backend.run() has been removed. Please see our migration guide "
            "https://quantum.cloud.ibm.com/docs/migration-guides/qiskit-runtime for instructions "
            "on how to migrate to the primitives interface."
        )

    def get_translation_stage_plugin(self) -> str:
        """Return the default translation stage plugin name for IBM backends."""
        if not self.options.use_fractional_gates:
            return "ibm_dynamic_circuits"
        return "ibm_dynamic_and_fractional"

    def status(self) -> BackendStatus:
        """Return the backend status.

        Returns:
            The status of the backend.

        """

        api_status = {
            "backend_name": self._configuration.backend_name,
            "backend_version": self._configuration.backend_version,
            "status_msg": "",
            "operational": self._qrmi.is_accessible(),
            "pending_jobs": 0,
        }

        return BackendStatus.from_dict(api_status)
