import json
from unittest.mock import MagicMock, patch
from dataclasses import dataclass

import pytest
from qiskit import QuantumCircuit
from qiskit_ibm_runtime.exceptions import IBMBackendError

from qrmi.primitives.ibm.backend import QRMIBackend, get_backend


class _FakeTarget:
    def __init__(self, value):
        self.value = value


class _FakeQRMI:
    def __init__(self, accessible=True):
        self._accessible = accessible

    def target(self):
        """Return a fake target object with JSON data."""
        return _FakeTarget(
            json.dumps(
                {
                    "configuration": {
                        "backend_name": "fake_backend",
                        "online_date": "2025-01-01T00:00:00Z",
                        "backend_version": "1.0.0",
                    },
                    "properties": {},
                }
            )
        )

    def is_accessible(self):
        """Return the accessibility status of the backend."""
        return self._accessible


@dataclass
class BackendFixture:
    """Dataclass to hold backend fixture data."""

    backend: QRMIBackend
    config: MagicMock
    props: MagicMock
    target: MagicMock
    mock_configuration: MagicMock
    mock_properties: MagicMock
    mock_convert: MagicMock


@pytest.fixture
def backend_fixture():
    """Fixture to create a QRMIBackend instance with mocked configuration and properties."""

    with (
        patch(
            "qrmi.primitives.ibm.backend.configuration_from_server_data"
        ) as mock_configuration,
        patch(
            "qrmi.primitives.ibm.backend.properties_from_server_data"
        ) as mock_properties,
        patch("qrmi.primitives.ibm.backend.convert_to_target") as mock_convert,
    ):
        config = MagicMock()
        config.backend_name = "fake_backend"
        config.backend_version = "1.0.0"
        config.dtm = 1e-9
        config.meas_map = [[0, 1]]
        config.custom_attribute = "custom_value"

        props = MagicMock()
        target = MagicMock()

        mock_configuration.return_value = config
        mock_properties.return_value = props
        mock_convert.return_value = target

        backend = QRMIBackend(_FakeQRMI())

        yield BackendFixture(
            backend=backend,
            config=config,
            props=props,
            target=target,
            mock_configuration=mock_configuration,
            mock_properties=mock_properties,
            mock_convert=mock_convert,
        )


def test_get_backend():
    """Verify QRMIBackend can be instantiated with a fake QRMI backend."""
    with (
        patch(
            "qrmi.primitives.ibm.backend.configuration_from_server_data"
        ) as mock_configuration,
        patch(
            "qrmi.primitives.ibm.backend.properties_from_server_data"
        ) as mock_properties,
        patch("qrmi.primitives.ibm.backend.convert_to_target") as mock_convert,
    ):
        config = MagicMock()
        config.backend_name = "fake_backend"
        config.backend_version = "1.0.0"

        mock_configuration.return_value = config
        mock_properties.return_value = MagicMock()
        mock_convert.return_value = MagicMock()

        qrmi_backend = _FakeQRMI()
        backend = get_backend(qrmi_backend)

        assert isinstance(backend, QRMIBackend)
        assert backend._qrmi is qrmi_backend
        assert backend.options.use_fractional_gates is False


def test_default_options():
    """Verify the expected default backend options."""

    options = QRMIBackend._default_options()

    assert options.shots == 4000
    assert options.memory is False
    assert options.memory_slots is None
    assert options.memory_slot_size == 100
    assert options.rep_time is None
    assert options.rep_delay is None
    assert options.init_qubits is True
    assert options.use_measure_esp is None
    assert options.use_fractional_gates is False
    assert options.noise_model is None
    assert options.seed_simulator is None


def test_configuration_returns_configuration(backend_fixture):
    """Verify the backend configuration is returned."""

    assert backend_fixture.backend.configuration() is backend_fixture.config


def test_properties_returns_properties(backend_fixture):
    """Verify cached backend properties are returned."""

    result = backend_fixture.backend.properties()

    assert result is backend_fixture.props
    assert backend_fixture.mock_properties.call_count == 1


def test_properties_refresh_reloads_properties(backend_fixture):
    """Verify backend properties are reloaded when refresh is requested."""

    backend_fixture.backend.properties(refresh=True)

    assert backend_fixture.mock_properties.call_count == 2


def test_properties_reloads_when_cache_is_missing(backend_fixture):
    """Verify backend properties are reloaded when no cached properties are available."""

    backend_fixture.backend._properties = None

    backend_fixture.backend.properties()

    assert backend_fixture.mock_properties.call_count == 2


def test_properties_updates_cached_properties(backend_fixture):
    """Verify reloaded backend properties are stored in cache."""

    new_properties = MagicMock()
    backend_fixture.mock_properties.return_value = new_properties

    backend_fixture.backend._properties = None

    result = backend_fixture.backend.properties()

    assert result is new_properties
    assert backend_fixture.backend._properties is new_properties


def test_properties_passes_fractional_gate_option(backend_fixture):
    """Verify the fractional-gates option is passed to the property decoder."""

    backend_fixture.backend.options.use_fractional_gates = True
    backend_fixture.backend._properties = None

    backend_fixture.backend.properties()

    assert (
        backend_fixture.mock_properties.call_args.kwargs["use_fractional_gates"] is True
    )


def test_target_returns_target(backend_fixture):
    """Verify the backend target is returned."""

    assert backend_fixture.backend.target is backend_fixture.target


def test_dtm_returns_configuration_value(backend_fixture):
    """Verify the dtm value from the backend configuration."""

    assert backend_fixture.backend.dtm == 1e-9


def test_meas_map_returns_configuration_value(backend_fixture):
    """Verify the measurement map from the backend configuration."""

    assert backend_fixture.backend.meas_map == [[0, 1]]


def test_max_circuits_returns_none(backend_fixture):
    """Verify None is returned for max_circuits."""

    assert backend_fixture.backend.max_circuits is None


def test_getattr_falls_back_to_configuration(backend_fixture):
    """Verify unknown attributes are resolved from backend configuration."""

    assert backend_fixture.backend.custom_attribute == "custom_value"


def test_getattr_raises_error_for_unknown_attribute(backend_fixture):
    """Verify backend raises AttributeError for unknown attributes."""

    with pytest.raises(AttributeError):
        backend_fixture.backend.__getattr__("non_existent_attribute")


@pytest.mark.parametrize(
    "attribute",
    [
        "_properties",
        "_target",
        "_configuration",
    ],
)
def test_getattr_raises_error_for_no_attribute(backend_fixture, attribute):
    """Verify backend raises AttributeError when no attribute is provided."""

    with pytest.raises(AttributeError):
        backend_fixture.backend.__getattr__(attribute)


def test_convert_to_target_skips_reload_target_exists(backend_fixture):
    """Verify _convert_to_target does not reload backend data when a target is already present."""

    backend_fixture.backend._target = backend_fixture.target

    backend_fixture.backend._convert_to_target()

    backend_fixture.mock_properties.assert_called_once()
    backend_fixture.mock_convert.assert_called_once()


def test_convert_to_target_refresh_reloads_backend_data(backend_fixture):
    """Verify _convert_to_target reloads configuration, properties and target when refresh is requested."""

    backend_fixture.backend._convert_to_target(refresh=True)

    assert backend_fixture.mock_configuration.call_count == 2
    assert backend_fixture.mock_properties.call_count == 2
    assert backend_fixture.mock_convert.call_count == 2


def test_convert_to_target_reloads_when_target_missing(backend_fixture):
    """Verify _convert_to_target reloads backend data when the cached target is unavailable."""

    backend_fixture.backend._target = None

    backend_fixture.backend._convert_to_target()

    assert backend_fixture.mock_configuration.call_count == 2
    assert backend_fixture.mock_properties.call_count == 2
    assert backend_fixture.mock_convert.call_count == 2


def test_convert_to_target_updates_cached_target(backend_fixture):
    """Verify _convert_to_target stores the newly generated target."""

    new_target = MagicMock()
    backend_fixture.mock_convert.return_value = new_target

    backend_fixture.backend._target = None

    backend_fixture.backend._convert_to_target()

    assert backend_fixture.backend._target is new_target


def test_convert_to_target_passes_fractional_gate(backend_fixture):
    """Verify _convert_to_target passes the fractional-gates option to backend converters."""

    backend_fixture.backend.options.use_fractional_gates = True
    backend_fixture.backend._target = None

    backend_fixture.backend._convert_to_target()

    assert (
        backend_fixture.mock_configuration.call_args.kwargs["use_fractional_gates"]
        is True
    )

    assert (
        backend_fixture.mock_properties.call_args.kwargs["use_fractional_gates"] is True
    )


def test_run_raises_backend_error(backend_fixture):
    """Verify IBMBackendError is raised when run() is called."""

    with pytest.raises(IBMBackendError):
        backend_fixture.backend.run()


def test_translation_plugin_without_fractional_gates(backend_fixture):
    """Verify the dynamic-circuits plugin is returned when fractional gates are disabled."""

    backend_fixture.backend.options.use_fractional_gates = False

    assert (
        backend_fixture.backend.get_translation_stage_plugin() == "ibm_dynamic_circuits"
    )


def test_translation_plugin_with_fractional_gates(backend_fixture):
    """Verify the fractional-gates plugin is returned when enabled."""

    backend_fixture.backend.options.use_fractional_gates = True

    assert (
        backend_fixture.backend.get_translation_stage_plugin()
        == "ibm_dynamic_and_fractional"
    )


@pytest.mark.parametrize(
    ("accessible", "expected"),
    [
        (True, True),
        (False, False),
    ],
)
def test_status_reports_backend_accessibility(accessible, expected):
    """Verify backend accessibility is reflected in status."""

    with (
        patch(
            "qrmi.primitives.ibm.backend.configuration_from_server_data"
        ) as mock_configuration,
        patch(
            "qrmi.primitives.ibm.backend.properties_from_server_data"
        ) as mock_properties,
        patch("qrmi.primitives.ibm.backend.convert_to_target") as mock_convert,
    ):
        config = MagicMock()
        config.backend_name = "fake_backend"
        config.backend_version = "1.0.0"

        mock_configuration.return_value = config
        mock_properties.return_value = MagicMock()
        mock_convert.return_value = MagicMock()

        backend = QRMIBackend(_FakeQRMI(accessible=accessible))

        status = backend.status()

        assert status.operational is expected


def test_check_faulty_qubit_raises_error(backend_fixture):
    """Verify ValueError is raised when a circuit uses a faulty qubit."""

    backend_fixture.props.faulty_qubits.return_value = [0]
    backend_fixture.props.faulty_gates.return_value = []

    qc = QuantumCircuit(1)
    qc.x(0)

    with pytest.raises(ValueError, match="faulty qubit"):
        backend_fixture.backend.check_faulty(qc)


def test_check_faulty_edge_raises_error(backend_fixture):
    """Verify ValueError is raised when a circuit uses a faulty edge."""

    gate = MagicMock()
    gate.qubits = [0, 1]

    backend_fixture.props.faulty_qubits.return_value = []
    backend_fixture.props.faulty_gates.return_value = [gate]

    qc = QuantumCircuit(2)
    qc.cx(0, 1)

    with pytest.raises(ValueError, match="faulty edge"):
        backend_fixture.backend.check_faulty(qc)


def test_check_faulty_ignores_barrier(backend_fixture):
    """Verify barrier instructions are ignored when checking for faults."""

    backend_fixture.props.faulty_qubits.return_value = []
    backend_fixture.props.faulty_gates.return_value = []

    qc = QuantumCircuit(2)
    qc.barrier()

    backend_fixture.backend.check_faulty(qc)


def test_check_faulty_returns_when_props_unavailable(backend_fixture):
    """Verify check_faulty returns without error when properties are unavailable."""

    qc = QuantumCircuit(1)
    qc.x(0)

    with patch.object(
        backend_fixture.backend,
        "properties",
        return_value=None,
    ) as mock_properties:
        backend_fixture.backend.check_faulty(qc)

    mock_properties.assert_called_once()


def test_refresh_calls_convert_to_target(backend_fixture):
    """Verify refresh calls _convert_to_target."""

    with patch.object(backend_fixture.backend, "_convert_to_target") as mock_convert:
        backend_fixture.backend.refresh()

    mock_convert.assert_called_once_with(refresh=True)
