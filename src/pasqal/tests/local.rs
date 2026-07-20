use super::qpu_slots_from_env;
use std::sync::{Mutex, OnceLock};

fn env_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
}

#[test]
fn qpu_slots_default_to_one() {
    let _guard = env_lock();
    std::env::remove_var("QRMI_JOB_QPU_SLOTS");
    assert_eq!(qpu_slots_from_env().unwrap(), 1);
}

#[test]
fn qpu_slots_read_from_env() {
    let _guard = env_lock();
    std::env::set_var("QRMI_JOB_QPU_SLOTS", "5");
    assert_eq!(qpu_slots_from_env().unwrap(), 5);
    std::env::remove_var("QRMI_JOB_QPU_SLOTS");
}

#[test]
fn qpu_slots_reject_zero() {
    let _guard = env_lock();
    std::env::set_var("QRMI_JOB_QPU_SLOTS", "0");
    assert!(qpu_slots_from_env().is_err());
    std::env::remove_var("QRMI_JOB_QPU_SLOTS");
}

#[test]
fn qpu_slots_reject_negative() {
    let _guard = env_lock();
    std::env::set_var("QRMI_JOB_QPU_SLOTS", "-1");
    assert!(qpu_slots_from_env().is_err());
    std::env::remove_var("QRMI_JOB_QPU_SLOTS");
}
