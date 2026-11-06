// This code is part of Qiskit.
//
// (C) Copyright Qoro Quantum 2026
//
// This code is licensed under the Apache License, Version 2.0. You may
// obtain a copy of this license in the LICENSE.txt file in the root directory
// of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
//
// Any modifications or derivative works of this code must retain this
// copyright notice, and modified files need to carry a notice indicating
// that they have been altered from the originals.

#![cfg(unix)]

use maestro_local_api::maestro::session;

#[test]
#[ignore = "requires a running Maestro server at /run/maestro.sock"]
fn session_task_execution_test() {
    let new_session = session::Session::new();
    assert!(
        new_session.is_ok(),
        "Failed to create new session: {:?}",
        new_session
    );

    let session = new_session.unwrap();
    let sess_id = session.get_id();
    let exists = session::Session::session_exists(sess_id);
    assert!(
        exists.is_ok(),
        "Failed to check if session exists: {:?}",
        exists
    );
    assert!(exists.unwrap(), "Session does not exist after creation");

    // the next id might exist, because there is another test creating a session, so add a safe value!
    // unless explicitely serialized, they will run in parallel!
    let next_id = sess_id + 2;
    let exists_next = session::Session::session_exists(next_id);
    assert!(
        exists_next.is_ok(),
        "Failed to check if next session exists: {:?}",
        exists_next
    );
    assert!(!exists_next.unwrap(), "Next session ID should not exist");

    // now delete the Session
    let result = session.delete();
    assert!(result.is_ok(), "Failed to delete session: {:?}", result);
    assert!(result.unwrap(), "Session deletion did not return OK");

    // it should not exist now
    let exists_after_delete = session::Session::session_exists(sess_id);
    assert!(
        exists_after_delete.is_ok(),
        "Failed to check if session exists after deletion: {:?}",
        exists_after_delete
    );
    assert!(
        !exists_after_delete.unwrap(),
        "Session still exists after deletion"
    );

    // create it again
    let session = session::Session::new().unwrap();

    let task_id = 0; // whatever

    let task_exists = session.task_exists(task_id);
    assert!(
        task_exists.is_ok(),
        "Failed to check if task exists: {:?}",
        task_exists
    );
    // should not exist

    assert!(
        !task_exists.unwrap(),
        "Task should not exist in new session"
    );

    let new_task = session.create_task();
    assert!(
        new_task.is_ok(),
        "Failed to create new task: {:?}",
        new_task
    );

    let task = new_task.unwrap();
    let task_exists = task.exists();
    assert!(
        task_exists.is_ok(),
        "Failed to check if task exists: {:?}",
        task_exists
    );
    assert!(task_exists.unwrap(), "Task should exist after creation");

    let task_valid = task.valid();
    assert!(
        task_valid.is_ok(),
        "Failed to check if task is valid: {:?}",
        task_valid
    );
    assert!(
        !task_valid.unwrap(),
        "Task should be invalid after creation"
    );

    let task_pending = task.pending();
    assert!(
        task_pending.is_ok(),
        "Failed to check if task is pending: {:?}",
        task_pending
    );
    assert!(
        !task_pending.unwrap(),
        "Task should not be pending after creation"
    );

    let task_running = task.running();
    assert!(
        task_running.is_ok(),
        "Failed to check if task is running: {:?}",
        task_running
    );
    assert!(
        !task_running.unwrap(),
        "Task should not be running after creation"
    );

    let task_finished = task.finished();
    assert!(
        task_finished.is_ok(),
        "Failed to check if task is finished: {:?}",
        task_finished
    );
    assert!(
        !task_finished.unwrap(),
        "Task should not be finished after creation"
    );

    let task_failed = task.failed();
    assert!(
        task_failed.is_ok(),
        "Failed to check if task is failed: {:?}",
        task_failed
    );
    assert!(
        !task_failed.unwrap(),
        "Task should not be failed after creation"
    );

    // try to run it, should fail
    let task_run = task.execute();
    assert!(task_run.is_err(), "Failed to run task: {:?}", task_run);

    let type_set = task.set_type(session::TaskType::EXECUTE);
    assert!(type_set.is_ok(), "Failed to set task type: {:?}", type_set);
    assert!(type_set.unwrap(), "Setting task type did not return OK");

    let qubits_set = task.set_qubits(5);
    assert!(
        qubits_set.is_ok(),
        "Failed to set task qubits: {:?}",
        qubits_set
    );
    assert!(qubits_set.unwrap(), "Setting task qubits did not return OK");

    let sim_set = task.set_simulator_type(1);
    assert!(
        sim_set.is_ok(),
        "Failed to set task simulator type: {:?}",
        sim_set
    );
    assert!(
        sim_set.unwrap(),
        "Setting task simulator type did not return OK"
    );

    let method_set = task.set_simulation_method(1);
    assert!(
        method_set.is_ok(),
        "Failed to set task simulation method: {:?}",
        method_set
    );
    assert!(
        method_set.unwrap(),
        "Setting task simulation method did not return OK"
    );

    let qasm_str = "OPENQASM 2.0; \
            creg c[3]; \
            qreg q[3]; \
            x q[0]; \
            cx q[0],q[1]; \
            measure q[0]->c[0]; \
            measure q[1]->c[1];"
        .to_string();

    let qasm_set = task.set_qasm(qasm_str);
    assert!(qasm_set.is_ok(), "Failed to set task QASM: {:?}", qasm_set);
    assert!(qasm_set.unwrap(), "Setting task QASM did not return OK");

    let mut config = session::TaskConfig::new();
    config.shots = 1000;

    let config_set = task.set_options(config);
    assert!(
        config_set.is_ok(),
        "Failed to set task config: {:?}",
        config_set
    );
    assert!(config_set.unwrap(), "Setting task config did not return OK");

    // now execution should work
    let task_run = task.execute();
    assert!(task_run.is_ok(), "Failed to run task: {:?}", task_run);
    assert!(task_run.unwrap(), "Running task did not return OK");

    let mut finished = false;
    while !finished {
        let task_finished = task.finished();
        assert!(
            task_finished.is_ok(),
            "Failed to check if task is finished: {:?}",
            task_finished
        );
        finished = task_finished.unwrap();
        if !finished {
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }

    let result = task.get_results();
    assert!(result.is_ok(), "Failed to get task result: {:?}", result);
    let result_content = result.unwrap();

    // Do something with result_content
    assert!(
        result_content.counts.contains_key("11000"),
        "Execute result counts does not have '11000' key"
    );
    assert!(
        result_content.counts["11000"] == 1000,
        "Execute result counts '11000' value is not 1"
    );
    assert!(
        result_content.simulator_type == "qcsim",
        "Execute result simulator is not 'qcsim'"
    );
    assert!(
        result_content.simulation_method == "matrix_product_state",
        "Execute result method is not 'matrix_product_state'"
    );

    // now the task should not exist anymore, as we got the results
    let task_exists = task.exists();
    assert!(
        task_exists.is_ok(),
        "Failed to check if task exists after getting results: {:?}",
        task_exists
    );
    assert!(
        !task_exists.unwrap(),
        "Task should not exist after getting results"
    );

    // delete the Session
    let result = session.delete();
    assert!(result.is_ok(), "Failed to delete session: {:?}", result);
    assert!(result.unwrap(), "Session deletion did not return OK");

    // it should not exist now
    let exists_after_delete = session::Session::session_exists(sess_id);
    assert!(
        exists_after_delete.is_ok(),
        "Failed to check if session exists after deletion: {:?}",
        exists_after_delete
    );
    assert!(
        !exists_after_delete.unwrap(),
        "Session still exists after deletion"
    );
}

#[test]
#[ignore = "requires a running Maestro server at /run/maestro.sock"]
fn estimation_test() {
    let new_session = session::Session::new();
    assert!(
        new_session.is_ok(),
        "Failed to create new session: {:?}",
        new_session
    );

    let session = new_session.unwrap();
    let sess_id = session.get_id();

    let new_task = session.create_task();
    assert!(
        new_task.is_ok(),
        "Failed to create new task: {:?}",
        new_task
    );

    let task = new_task.unwrap();

    let type_set = task.set_type(session::TaskType::ESTIMATE);
    assert!(type_set.is_ok(), "Failed to set task type: {:?}", type_set);
    assert!(type_set.unwrap(), "Setting task type did not return OK");

    let qubits_set = task.set_qubits(5);
    assert!(
        qubits_set.is_ok(),
        "Failed to set task qubits: {:?}",
        qubits_set
    );
    assert!(qubits_set.unwrap(), "Setting task qubits did not return OK");

    let sim_set = task.set_simulator_type(0);
    assert!(
        sim_set.is_ok(),
        "Failed to set task simulator type: {:?}",
        sim_set
    );
    assert!(
        sim_set.unwrap(),
        "Setting task simulator type did not return OK"
    );

    let method_set = task.set_simulation_method(0);
    assert!(
        method_set.is_ok(),
        "Failed to set task simulation method: {:?}",
        method_set
    );
    assert!(
        method_set.unwrap(),
        "Setting task simulation method did not return OK"
    );

    let qasm_str = "OPENQASM 2.0; \
            creg c[3]; \
            qreg q[3]; \
            x q[0]; \
            cx q[0],q[1];"
        .to_string();

    let qasm_set = task.set_qasm(qasm_str);
    assert!(qasm_set.is_ok(), "Failed to set task QASM: {:?}", qasm_set);
    assert!(qasm_set.unwrap(), "Setting task QASM did not return OK");

    // set pauli strings
    let pauli_set = task.set_observables(vec![
        "zzz".to_string(),
        "yyy".to_string(),
        "xxx".to_string(),
    ]);
    assert!(
        pauli_set.is_ok(),
        "Failed to set task observables: {:?}",
        pauli_set
    );
    assert!(
        pauli_set.unwrap(),
        "Setting task observables did not return OK"
    );

    // now execution should work
    let task_run = task.execute();
    assert!(task_run.is_ok(), "Failed to run task: {:?}", task_run);
    assert!(task_run.unwrap(), "Running task did not return OK");

    let mut finished = false;
    while !finished {
        let task_finished = task.finished();
        assert!(
            task_finished.is_ok(),
            "Failed to check if task is finished: {:?}",
            task_finished
        );
        finished = task_finished.unwrap();
        if !finished {
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }

    let result = task.get_results();
    assert!(result.is_ok(), "Failed to get task result: {:?}", result);
    let result_content = result.unwrap();

    // Do something with result_content
    assert!(
        result_content.expectation_values.len() == 3,
        "There should be 3 expectation values, received {}",
        result_content.expectation_values.len()
    );

    for (i, val) in result_content.expectation_values.into_iter().enumerate() {
        if i == 0 {
            assert!(
                (val - 1.0).abs() < 1e-5,
                "Expectation value for 'zzz' is not 1.0, got {}",
                val
            );
        } else if i == 1 {
            assert!(
                val.abs() < 1e-5,
                "Expectation value for 'yyy' is not 0.0, got {}",
                val
            );
        } else if i == 2 {
            assert!(
                val.abs() < 1e-5,
                "Expectation value for 'xxx' is not 0.0, got {}",
                val
            );
        }
        assert!(val.is_finite(), "Expectation value is not finite: {}", val);
    }

    assert!(
        result_content.simulator_type == "aer",
        "Execute result simulator is not 'aer'"
    );
    assert!(
        result_content.simulation_method == "statevector",
        "Execute result method is not 'statevector'"
    );

    // now the task should not exist anymore, as we got the results
    let task_exists = task.exists();
    assert!(
        task_exists.is_ok(),
        "Failed to check if task exists after getting results: {:?}",
        task_exists
    );
    assert!(
        !task_exists.unwrap(),
        "Task should not exist after getting results"
    );

    // delete the Session
    let result = session.delete();
    assert!(result.is_ok(), "Failed to delete session: {:?}", result);
    assert!(result.unwrap(), "Session deletion did not return OK");

    // it should not exist now
    let exists_after_delete = session::Session::session_exists(sess_id);
    assert!(
        exists_after_delete.is_ok(),
        "Failed to check if session exists after deletion: {:?}",
        exists_after_delete
    );
    assert!(
        !exists_after_delete.unwrap(),
        "Session still exists after deletion"
    );
}
