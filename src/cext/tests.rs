use super::*;
use crate::maestro::MaestroLocal;

#[test]
fn maestro_payload_rejects_invalid_utf8_before_submission() {
    let mut resource = QuantumResource {
        inner: Box::new(MaestroLocal::from_config("c_maestro", Default::default()).unwrap()),
        runtime: Arc::new(tokio::runtime::Runtime::new().unwrap()),
    };
    let invalid = CString::new(vec![0xff]).unwrap();
    for field in 0..4 {
        let mut strings = [c"OPENQASM 2.0; qreg q[1];", c"execute", c"", c"{}"];
        strings[field] = invalid.as_c_str();
        let payload = Payload::MaestroLocal {
            input: strings[0].as_ptr().cast_mut(),
            job_type: strings[1].as_ptr().cast_mut(),
            qubits: 1,
            simulator_type: 0,
            simulation_method: 0,
            observables: strings[2].as_ptr().cast_mut(),
            config: strings[3].as_ptr().cast_mut(),
        };
        let mut task_id = std::ptr::null_mut();
        unsafe {
            assert!(matches!(
                qrmi_resource_task_start(&mut resource, &payload, &mut task_id),
                ReturnCode::InvalidInputError
            ));
            let message = qrmi_get_last_error();
            assert!(!message.is_null());
            assert!(CStr::from_ptr(message).to_str().unwrap().contains("UTF-8"));
            qrmi_string_free(message);
        }
        assert!(task_id.is_null());
    }
    unsafe {
        let mut task_id = std::ptr::null_mut();
        assert!(matches!(
            qrmi_resource_task_start(&mut resource, std::ptr::null(), &mut task_id),
            ReturnCode::NullPointerError
        ));
        assert!(task_id.is_null());
    }
}

#[test]
fn resource_type_parser_keeps_maestro_and_upstream_values() {
    for (name, expected) in [
        (c"maestro-local", ResourceType::MaestroLocal),
        (c"oqtopus", ResourceType::OQTOPUS),
        (c"pasqal-local", ResourceType::PasqalLocal),
    ] {
        let mut actual = ResourceType::IBMQuantumSystem;
        unsafe {
            assert!(matches!(
                qrmi_config_resource_type_from_str(name.as_ptr(), &mut actual),
                ReturnCode::Success
            ));
        }
        assert_eq!(actual, expected);
    }
}
