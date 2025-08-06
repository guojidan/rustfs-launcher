use crate::config::RustFsConfig;
use crate::error::{Error, Result};
use crate::process;
use crate::state;

#[tauri::command]
pub async fn launch_rustfs(config: RustFsConfig) -> Result<String> {
    process::launch(config)
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
pub async fn diagnose_rustfs_binary() -> Result<String> {
    process::diagnose_binary()
}

#[tauri::command]
pub async fn get_app_logs() -> Result<Vec<String>> {
    Ok(state::get_app_logs())
}

#[tauri::command]
pub async fn get_rustfs_logs() -> Result<Vec<String>> {
    Ok(state::get_rustfs_logs())
}
