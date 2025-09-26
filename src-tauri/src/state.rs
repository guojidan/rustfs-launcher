use lazy_static::lazy_static;
use regex::Regex;
use std::collections::VecDeque;
use std::process::Child;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager};

lazy_static! {
    pub static ref APP_LOGS: Arc<Mutex<VecDeque<String>>> = Arc::new(Mutex::new(VecDeque::new()));
    pub static ref RUSTFS_LOGS: Arc<Mutex<VecDeque<String>>> =
        Arc::new(Mutex::new(VecDeque::new()));
    pub static ref APP_HANDLE: Arc<Mutex<Option<AppHandle>>> = Arc::new(Mutex::new(None));
    pub static ref RUSTFS_PROCESS: Arc<Mutex<Option<Child>>> = Arc::new(Mutex::new(None));
}

lazy_static! {
    static ref ANSI_REGEX: Regex = Regex::new(r"\x1B\[[0-9;]*m").unwrap();
}

fn clean_ansi_codes(s: &str) -> String {
    ANSI_REGEX.replace_all(s, "").to_string()
}

fn buffer_log(logs: &Arc<Mutex<VecDeque<String>>>, message: String, capacity: usize) -> String {
    let cleaned_message = clean_ansi_codes(&message);
    let log_entry = format!(
        "[{}] {}",
        chrono::Local::now().format("%H:%M:%S"),
        cleaned_message
    );

    {
        let mut logs = logs.lock().unwrap();
        logs.push_back(log_entry.clone());
        if logs.len() > capacity {
            logs.pop_front();
        }
    }

    log_entry
}

fn emit_log(event_name: &str, log_entry: String) {
    if let Some(handle) = APP_HANDLE.lock().unwrap().as_ref() {
        if let Some(window) = handle.get_webview_window("main") {
            let _ = window.emit(event_name, log_entry);
        }
    }
}

const APP_LOG_EVENT: &str = "app-log";
const RUSTFS_LOG_EVENT: &str = "rustfs-log";
const APP_LOG_CAPACITY: usize = 100;
const RUSTFS_LOG_CAPACITY: usize = 1000;

pub fn add_app_log(message: String) {
    let entry = buffer_log(&APP_LOGS, message, APP_LOG_CAPACITY);
    emit_log(APP_LOG_EVENT, entry);
}

pub fn add_rustfs_log(message: String) {
    let entry = buffer_log(&RUSTFS_LOGS, message, RUSTFS_LOG_CAPACITY);
    emit_log(RUSTFS_LOG_EVENT, entry);
}

pub fn set_app_handle(handle: AppHandle) {
    *APP_HANDLE.lock().unwrap() = Some(handle);
}

pub fn get_app_logs() -> Vec<String> {
    APP_LOGS.lock().unwrap().iter().cloned().collect()
}

pub fn get_rustfs_logs() -> Vec<String> {
    RUSTFS_LOGS.lock().unwrap().iter().cloned().collect()
}

pub fn set_rustfs_process(process: Child) {
    let pid = process.id();
    *RUSTFS_PROCESS.lock().unwrap() = Some(process);
    add_app_log(format!("RustFS process registered with PID: {}", pid));
}

pub fn terminate_rustfs_process() {
    let mut process_guard = RUSTFS_PROCESS.lock().unwrap();
    if let Some(mut process) = process_guard.take() {
        let pid = process.id();
        add_app_log(format!("Terminating RustFS process with PID: {}", pid));

        match process.kill() {
            Ok(_) => {
                add_app_log("RustFS process terminated successfully".to_string());
                // Wait for the process to actually exit
                let _ = process.wait();
            }
            Err(e) => {
                add_app_log(format!("Failed to terminate RustFS process: {}", e));
            }
        }
    } else {
        add_app_log("No RustFS process to terminate".to_string());
    }
}
