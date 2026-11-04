use crate::maestro_lib;
use json;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug)]
pub enum TaskType {
    EXECUTE,
    ESTIMATE,
}

#[derive(Debug)]
pub struct Task {
    pub id: u32,
    pub session_id: u32,
}

#[derive(Debug)]
pub struct Session {
    pub id: u32,
}

impl Session {
    pub fn new() -> Result<Self, String> {
        Self::new_at(&maestro_lib::socket_path())
    }

    pub fn new_at(socket: &Path) -> Result<Self, String> {
        let res = maestro_lib::send_command_at(socket, "SESSION CREATE\n");
        match res {
            maestro_lib::Response::OkResponse(id_str) => {
                if let Ok(id) = id_str.parse::<u32>() {
                    Ok(Session { id })
                } else {
                    Err(format!(
                        "Failed to parse session ID from response: {}",
                        id_str
                    ))
                }
            }
            maestro_lib::Response::ERROR(err) => Err(format!("Failed to create session: {}", err)),
            _ => Err("Unexpected response when creating session".to_string()),
        }
    }

    pub fn get_id(&self) -> u32 {
        self.id
    }

    pub async fn session_exists(session_id: u32) -> Result<bool, String> {
        Self::session_exists_at(session_id, &maestro_lib::socket_path()).await
    }

    pub async fn session_exists_at(session_id: u32, socket: &Path) -> Result<bool, String> {
        let command = format!("SESSION {} EXISTS\n", session_id);
        let res = maestro_lib::send_command_at(socket, &command);
        match res {
            maestro_lib::Response::OK(exists) => Ok(exists),
            maestro_lib::Response::ERROR(err) => {
                Err(format!("Failed to check if session exists: {}", err))
            }
            _ => Err("Unexpected response when checking if session exists".to_string()),
        }
    }

    pub async fn delete(&self) -> Result<bool, String> {
        self.delete_at(&maestro_lib::socket_path()).await
    }

    pub async fn delete_at(&self, socket: &Path) -> Result<bool, String> {
        let command = format!("SESSION {} DELETE\n", self.id);
        let res = maestro_lib::send_command_at(socket, &command);
        match res {
            maestro_lib::Response::OK(val) => Ok(val),
            maestro_lib::Response::ERROR(err) => Err(format!("Failed to delete session: {}", err)),
            _ => Err("Unexpected response when deleting session".to_string()),
        }
    }

    pub async fn task_exists(&self, task_id: u32) -> Result<bool, String> {
        self.task_exists_at(task_id, &maestro_lib::socket_path())
            .await
    }

    pub async fn task_exists_at(&self, task_id: u32, socket: &Path) -> Result<bool, String> {
        let command = format!("SESSION {} TASK {} EXISTS\n", self.id, task_id);
        let res = maestro_lib::send_command_at(socket, &command);
        match res {
            maestro_lib::Response::OK(exists) => Ok(exists),
            maestro_lib::Response::ERROR(err) => {
                Err(format!("Failed to check if task exists: {}", err))
            }
            _ => Err("Unexpected response when checking if task exists".to_string()),
        }
    }

    pub async fn create_task(&self) -> Result<Task, String> {
        self.create_task_at(&maestro_lib::socket_path()).await
    }

    pub async fn create_task_at(&self, socket: &Path) -> Result<Task, String> {
        let command = format!("SESSION {} TASK CREATE\n", self.id);
        let res = maestro_lib::send_command_at(socket, &command);
        match res {
            maestro_lib::Response::OkResponse(id_str) => {
                if let Ok(id) = id_str.parse::<u32>() {
                    Ok(Task {
                        id,
                        session_id: self.id,
                    })
                } else {
                    Err(format!("Failed to parse task ID from response: {}", id_str))
                }
            }
            maestro_lib::Response::ERROR(err) => Err(format!("Failed to create task: {}", err)),
            _ => Err("Unexpected response when creating task".to_string()),
        }
    }
}

#[derive(Debug)]
pub struct TaskResult {
    pub task_id: u32,
    pub session_id: u32,
    pub time_taken: f64,
    pub simulator_type: String,
    pub simulation_method: String,
    pub counts: HashMap<String, u64>,
    pub expectation_values: Vec<f64>,
}

#[derive(Debug)]
pub struct TaskConfig {
    pub matrix_product_state_max_bond_dimension: u32,
    pub matrix_product_state_truncation_threshold: f64,
    pub mps_sample_measure_algorithm: String,
    pub shots: u64,
}

impl Default for TaskConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskConfig {
    pub fn new() -> Self {
        Self {
            matrix_product_state_max_bond_dimension: 0,
            matrix_product_state_truncation_threshold: 0.0,
            mps_sample_measure_algorithm: String::new(),
            shots: 1,
        }
    }

    fn get_json(&self) -> String {
        let mut json_obj = json::JsonValue::new_object();

        if self.matrix_product_state_max_bond_dimension > 0 {
            json_obj["matrix_product_state_max_bond_dimension"] =
                self.matrix_product_state_max_bond_dimension.into();
        }

        if self.matrix_product_state_truncation_threshold > 0.0 {
            json_obj["matrix_product_state_truncation_threshold"] =
                self.matrix_product_state_truncation_threshold.into();
        }

        if !self.mps_sample_measure_algorithm.is_empty() {
            json_obj["mps_sample_measure_algorithm"] =
                self.mps_sample_measure_algorithm.clone().into();
        }

        json_obj["shots"] = self.shots.into();

        json_obj.dump()
    }
}

impl Task {
    pub fn get_id(&self) -> u32 {
        self.id
    }

    pub fn get_session_id(&self) -> u32 {
        self.session_id
    }

    pub async fn exists(&self) -> Result<bool, String> {
        self.exists_at(&maestro_lib::socket_path()).await
    }

    pub async fn exists_at(&self, socket: &Path) -> Result<bool, String> {
        let command = format!("SESSION {} TASK {} EXISTS\n", self.session_id, self.id);
        let res = maestro_lib::send_command_at(socket, &command);
        match res {
            maestro_lib::Response::OK(exists) => Ok(exists),
            maestro_lib::Response::ERROR(err) => {
                Err(format!("Failed to check if task exists: {}", err))
            }
            _ => Err("Unexpected response when checking if task exists".to_string()),
        }
    }

    pub async fn valid(&self) -> Result<bool, String> {
        self.valid_at(&maestro_lib::socket_path()).await
    }

    pub async fn valid_at(&self, socket: &Path) -> Result<bool, String> {
        let command = format!("SESSION {} TASK {} VALID\n", self.session_id, self.id);
        let res = maestro_lib::send_command_at(socket, &command);
        match res {
            maestro_lib::Response::OK(valid) => Ok(valid),
            maestro_lib::Response::ERROR(err) => {
                Err(format!("Failed to check task validity: {}", err))
            }
            _ => Err("Unexpected response when checking task validity".to_string()),
        }
    }

    pub async fn pending(&self) -> Result<bool, String> {
        self.pending_at(&maestro_lib::socket_path()).await
    }

    pub async fn pending_at(&self, socket: &Path) -> Result<bool, String> {
        let command = format!("SESSION {} TASK {} PENDING\n", self.session_id, self.id);
        let res = maestro_lib::send_command_at(socket, &command);
        match res {
            maestro_lib::Response::OK(pending) => Ok(pending),
            maestro_lib::Response::ERROR(err) => {
                Err(format!("Failed to check if task is pending: {}", err))
            }
            _ => Err("Unexpected response when checking if task is pending".to_string()),
        }
    }

    pub async fn running(&self) -> Result<bool, String> {
        self.running_at(&maestro_lib::socket_path()).await
    }

    pub async fn running_at(&self, socket: &Path) -> Result<bool, String> {
        let command = format!("SESSION {} TASK {} RUNNING\n", self.session_id, self.id);
        let res = maestro_lib::send_command_at(socket, &command);
        match res {
            maestro_lib::Response::OK(running) => Ok(running),
            maestro_lib::Response::ERROR(err) => {
                Err(format!("Failed to check if task is running: {}", err))
            }
            _ => Err("Unexpected response when checking if task is running".to_string()),
        }
    }

    pub async fn finished(&self) -> Result<bool, String> {
        self.finished_at(&maestro_lib::socket_path()).await
    }

    pub async fn finished_at(&self, socket: &Path) -> Result<bool, String> {
        let command = format!("SESSION {} TASK {} FINISHED\n", self.session_id, self.id);
        let res = maestro_lib::send_command_at(socket, &command);
        match res {
            maestro_lib::Response::OK(finished) => Ok(finished),
            maestro_lib::Response::ERROR(err) => {
                Err(format!("Failed to check if task is finished: {}", err))
            }
            _ => Err("Unexpected response when checking if task is finished".to_string()),
        }
    }

    pub async fn failed(&self) -> Result<bool, String> {
        self.failed_at(&maestro_lib::socket_path()).await
    }

    pub async fn failed_at(&self, socket: &Path) -> Result<bool, String> {
        let command = format!("SESSION {} TASK {} FAILED\n", self.session_id, self.id);
        let res = maestro_lib::send_command_at(socket, &command);
        match res {
            maestro_lib::Response::OK(failed) => Ok(failed),
            maestro_lib::Response::ERROR(err) => {
                Err(format!("Failed to check if task is failed: {}", err))
            }
            _ => Err("Unexpected response when checking if task is failed".to_string()),
        }
    }

    pub async fn execute(&self) -> Result<bool, String> {
        self.execute_at(&maestro_lib::socket_path()).await
    }

    pub async fn execute_at(&self, socket: &Path) -> Result<bool, String> {
        let command = format!("SESSION {} TASK {} EXECUTE\n", self.session_id, self.id);
        let res = maestro_lib::send_command_at(socket, &command);
        match res {
            maestro_lib::Response::OK(executed) => Ok(executed),
            maestro_lib::Response::ERROR(err) => Err(format!("Failed to execute task: {}", err)),
            _ => Err("Unexpected response when executing task".to_string()),
        }
    }

    pub async fn cancel(&self) -> Result<bool, String> {
        self.cancel_at(&maestro_lib::socket_path()).await
    }

    pub async fn cancel_at(&self, socket: &Path) -> Result<bool, String> {
        let command = format!("SESSION {} TASK {} CANCEL\n", self.session_id, self.id);
        let res = maestro_lib::send_command_at(socket, &command);
        match res {
            maestro_lib::Response::OK(canceled) => Ok(canceled),
            maestro_lib::Response::ERROR(err) => Err(format!("Failed to cancel task: {}", err)),
            _ => Err("Unexpected response when canceling task".to_string()),
        }
    }

    fn get_task_results_from_string(&self, results: String) -> TaskResult {
        let parsed = json::parse(&results).unwrap();

        let time_taken = parsed["time_taken"].as_f64().unwrap_or(0.0);
        let simulator_type = parsed["simulator"].as_str().unwrap_or("").to_string();
        let simulation_method = parsed["method"].as_str().unwrap_or("").to_string();

        let mut counts = HashMap::<String, u64>::new();

        for (key, value) in parsed["counts"].entries() {
            if let Some(count) = value.as_u64() {
                counts.insert(key.to_string(), count);
            }
        }

        let mut expectation_values = Vec::<f64>::new();

        for val in parsed["expectation_values"].members() {
            if let Some(v) = val.as_f64() {
                expectation_values.push(v);
            }
        }

        TaskResult {
            task_id: self.id,
            session_id: self.session_id,
            time_taken,
            simulator_type,
            simulation_method,
            counts,
            expectation_values,
        }
    }

    pub async fn get_results_as_string(&self) -> Result<String, String> {
        self.get_results_as_string_at(&maestro_lib::socket_path())
            .await
    }

    pub async fn get_results_as_string_at(&self, socket: &Path) -> Result<String, String> {
        let command = format!("SESSION {} TASK {} GET_RESULTS\n", self.session_id, self.id);
        let res = maestro_lib::send_command_at(socket, &command);
        match res {
            maestro_lib::Response::OkResponse(results) => Ok(results),
            maestro_lib::Response::ERROR(err) => {
                Err(format!("Failed to get task results: {}", err))
            }
            _ => Err("Unexpected response when getting task results".to_string()),
        }
    }

    pub async fn get_results(&self) -> Result<TaskResult, String> {
        self.get_results_at(&maestro_lib::socket_path()).await
    }

    pub async fn get_results_at(&self, socket: &Path) -> Result<TaskResult, String> {
        let results = self.get_results_as_string_at(socket).await;

        if let Ok(result) = results {
            Ok(self.get_task_results_from_string(result))
        } else {
            Err(format!(
                "Failed to get task results: {}",
                results.unwrap_err()
            ))
        }
    }

    pub async fn set_type(&self, task_type: TaskType) -> Result<bool, String> {
        self.set_type_at(task_type, &maestro_lib::socket_path())
            .await
    }

    pub async fn set_type_at(&self, task_type: TaskType, socket: &Path) -> Result<bool, String> {
        let type_str = match task_type {
            TaskType::EXECUTE => "EXECUTE",
            TaskType::ESTIMATE => "ESTIMATE",
        };
        let command = format!(
            "SESSION {} TASK {} SET_TYPE {}\n",
            self.session_id, self.id, type_str
        );
        let res = maestro_lib::send_command_at(socket, &command);
        match res {
            maestro_lib::Response::OK(set) => Ok(set),
            maestro_lib::Response::ERROR(err) => Err(format!("Failed to set task type: {}", err)),
            _ => Err("Unexpected response when setting task type".to_string()),
        }
    }

    pub async fn set_qubits(&self, qubits: u32) -> Result<bool, String> {
        self.set_qubits_at(qubits, &maestro_lib::socket_path())
            .await
    }

    pub async fn set_qubits_at(&self, qubits: u32, socket: &Path) -> Result<bool, String> {
        let command = format!(
            "SESSION {} TASK {} SET_QUBITS {}\n",
            self.session_id, self.id, qubits
        );
        let res = maestro_lib::send_command_at(socket, &command);
        match res {
            maestro_lib::Response::OK(set) => Ok(set),
            maestro_lib::Response::ERROR(err) => Err(format!("Failed to set task qubits: {}", err)),
            _ => Err("Unexpected response when setting task qubits".to_string()),
        }
    }

    pub async fn set_simulator_type(&self, simulator_type: u32) -> Result<bool, String> {
        self.set_simulator_type_at(simulator_type, &maestro_lib::socket_path())
            .await
    }

    pub async fn set_simulator_type_at(
        &self,
        simulator_type: u32,
        socket: &Path,
    ) -> Result<bool, String> {
        let command = format!(
            "SESSION {} TASK {} SET_SIM_TYPE {}\n",
            self.session_id, self.id, simulator_type
        );
        let res = maestro_lib::send_command_at(socket, &command);
        match res {
            maestro_lib::Response::OK(set) => Ok(set),
            maestro_lib::Response::ERROR(err) => {
                Err(format!("Failed to set task simulator type: {}", err))
            }
            _ => Err("Unexpected response when setting task simulator type".to_string()),
        }
    }

    pub async fn set_simulation_method(&self, method: u32) -> Result<bool, String> {
        self.set_simulation_method_at(method, &maestro_lib::socket_path())
            .await
    }

    pub async fn set_simulation_method_at(
        &self,
        method: u32,
        socket: &Path,
    ) -> Result<bool, String> {
        let command = format!(
            "SESSION {} TASK {} SET_METHOD {}\n",
            self.session_id, self.id, method
        );
        let res = maestro_lib::send_command_at(socket, &command);
        match res {
            maestro_lib::Response::OK(set) => Ok(set),
            maestro_lib::Response::ERROR(err) => {
                Err(format!("Failed to set task simulation method: {}", err))
            }
            _ => Err("Unexpected response when setting task simulation method".to_string()),
        }
    }

    pub async fn set_observables_as_string(&self, observables: String) -> Result<bool, String> {
        self.set_observables_as_string_at(observables, &maestro_lib::socket_path())
            .await
    }

    pub async fn set_observables_as_string_at(
        &self,
        observables: String,
        socket: &Path,
    ) -> Result<bool, String> {
        // SET_OBSERVABLES is sent over a line-based socket protocol, so any
        // embedded newlines must be collapsed rather than merely trimmed from
        // the ends, or the command gets truncated at the first newline.
        let observables_trimmed = observables.replace(['\n', '\r'], " ");
        let observables_trimmed = observables_trimmed.trim();

        let command = format!(
            "SESSION {} TASK {} SET_OBSERVABLES {}\n",
            self.session_id, self.id, observables_trimmed
        );
        let res = maestro_lib::send_command_at(socket, &command);
        match res {
            maestro_lib::Response::OK(set) => Ok(set),
            maestro_lib::Response::ERROR(err) => {
                Err(format!("Failed to set task observables: {}", err))
            }
            _ => Err("Unexpected response when setting task observables".to_string()),
        }
    }

    pub async fn set_observables(&self, observables: Vec<String>) -> Result<bool, String> {
        self.set_observables_at(observables, &maestro_lib::socket_path())
            .await
    }

    pub async fn set_observables_at(
        &self,
        observables: Vec<String>,
        socket: &Path,
    ) -> Result<bool, String> {
        let observables_str = observables.join(";");
        return self
            .set_observables_as_string_at(observables_str, socket)
            .await;
    }

    pub async fn set_qasm(&self, qasm: String) -> Result<bool, String> {
        self.set_qasm_at(qasm, &maestro_lib::socket_path()).await
    }

    pub async fn set_qasm_at(&self, qasm: String, socket: &Path) -> Result<bool, String> {
        // SET_QASM is sent over a line-based socket protocol. QASM generated by
        // tools like Qiskit's qasm2.dumps() is multi-line, so embedded newlines
        // must be collapsed to spaces (statements are ';'-terminated, so this is
        // semantically safe) rather than merely trimmed from the ends, or the
        // command gets truncated at the first newline and the circuit is lost.
        let qasm_trimmed = qasm.replace(['\n', '\r'], " ");
        let qasm_trimmed = qasm_trimmed.trim();
        let command = format!(
            "SESSION {} TASK {} SET_QASM {}\n",
            self.session_id, self.id, qasm_trimmed
        );
        let res = maestro_lib::send_command_at(socket, &command);
        match res {
            maestro_lib::Response::OK(set) => Ok(set),
            maestro_lib::Response::ERROR(err) => Err(format!("Failed to set task QASM: {}", err)),
            _ => Err("Unexpected response when setting task QASM".to_string()),
        }
    }

    pub async fn set_options_json(&self, options: String) -> Result<bool, String> {
        self.set_options_json_at(options, &maestro_lib::socket_path())
            .await
    }

    pub async fn set_options_json_at(
        &self,
        options: String,
        socket: &Path,
    ) -> Result<bool, String> {
        // SET_OPTIONS is sent over a line-based socket protocol, so any
        // embedded newlines (e.g. from pretty-printed JSON) must be collapsed
        // rather than merely trimmed from the ends, or the command gets
        // truncated at the first newline.
        let options_trimmed = options.replace(['\n', '\r'], " ");
        let options_trimmed = options_trimmed.trim();
        let command = format!(
            "SESSION {} TASK {} SET_OPTIONS {}\n",
            self.session_id, self.id, options_trimmed
        );
        let res = maestro_lib::send_command_at(socket, &command);
        match res {
            maestro_lib::Response::OK(set) => Ok(set),
            maestro_lib::Response::ERROR(err) => {
                Err(format!("Failed to set task options: {}", err))
            }
            _ => Err("Unexpected response when setting task options".to_string()),
        }
    }

    pub async fn set_options(&self, options: TaskConfig) -> Result<bool, String> {
        self.set_options_at(options, &maestro_lib::socket_path())
            .await
    }

    pub async fn set_options_at(&self, options: TaskConfig, socket: &Path) -> Result<bool, String> {
        let json_str = options.get_json();

        self.set_options_json_at(json_str, socket).await
    }
}
