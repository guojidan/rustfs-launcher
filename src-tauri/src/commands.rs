use crate::config::RustFsConfig;
use crate::error::{Error, Result};
use crate::process;
use crate::state;
use serde::Serialize;
use std::io::{Error as IoError, ErrorKind};
use tauri::async_runtime;

#[derive(Debug, Serialize)]
pub struct CommandResponse {
    pub success: bool,
    pub message: String,
}

#[tauri::command]
pub async fn launch_rustfs(config: RustFsConfig) -> Result<CommandResponse> {
    let handle = async_runtime::spawn_blocking(move || process::launch(config));
    let message = handle.await.map_err(|err| {
        let io_error = IoError::new(ErrorKind::Other, err.to_string());
        Error::Io(io_error)
    })??;

    Ok(CommandResponse {
        success: true,
        message,
    })
}

#[tauri::command]
pub async fn validate_config(config: RustFsConfig) -> Result<bool> {
    if config.data_path.is_empty() {
        return Err(Error::DataPathRequired);
    }
    if !std::path::Path::new(&config.data_path).exists() {
        return Err(Error::DataPathNotExist(config.data_path));
    }
    Ok(true)
}

#[tauri::command]
pub async fn diagnose_rustfs_binary() -> Result<CommandResponse> {
    let handle = async_runtime::spawn_blocking(process::diagnose_binary);
    let message = handle.await.map_err(|err| {
        let io_error = IoError::new(ErrorKind::Other, err.to_string());
        Error::Io(io_error)
    })??;

    Ok(CommandResponse {
        success: true,
        message,
    })
}

#[tauri::command]
pub async fn get_app_logs() -> Result<Vec<String>> {
    Ok(state::get_app_logs())
}

#[tauri::command]
pub async fn get_rustfs_logs() -> Result<Vec<String>> {
    Ok(state::get_rustfs_logs())
}
