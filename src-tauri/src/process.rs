use crate::config::RustFsConfig;
use crate::error::{Error, Result};
use crate::state::{add_app_log, add_rustfs_log, set_rustfs_process};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;

fn get_binary_path() -> Result<PathBuf> {
    let current_exe = std::env::current_exe().map_err(Error::Io)?;
    let exe_dir = current_exe.parent().ok_or_else(|| {
        Error::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Parent directory of executable not found",
        ))
    })?;
    let binaries_dir = exe_dir.join("binaries");

    let binary_name = if cfg!(target_os = "macos") {
        "rustfs-macos-aarch64"
    } else if cfg!(target_os = "windows") {
        "rustfs-windows-x86_64.exe"
    } else {
        "rustfs"
    };

    let binary_path = binaries_dir.join(binary_name);
    add_app_log(format!("Resolved binary path: {}", binary_path.display()));

    if !binary_path.exists() {
        return Err(Error::BinaryNotFound(
            binary_path.to_string_lossy().to_string(),
        ));
    }

    Ok(binary_path)
}

#[cfg(unix)]
fn check_permissions(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let metadata = std::fs::metadata(path)
        .map_err(|e| Error::Metadata(path.to_string_lossy().to_string(), e))?;
    let permissions = metadata.permissions();
    add_app_log(format!(
        "File permissions for {}: {:o}",
        path.display(),
        permissions.mode()
    ));

    if permissions.mode() & 0o111 == 0 {
        add_app_log("WARNING: Binary is not executable".to_string());
    }
    Ok(())
}

#[cfg(not(unix))]
fn check_permissions(path: &Path) -> Result<()> {
    let metadata = std::fs::metadata(path)
        .map_err(|e| Error::Metadata(path.to_string_lossy().to_string(), e))?;
    
    add_app_log(format!("File size: {} bytes", metadata.len()));
    
    // Check if file is readable
    if metadata.permissions().readonly() {
        add_app_log("WARNING: Binary file is read-only".to_string());
    }
    
    // Check if it's a regular file
    if !metadata.is_file() {
        return Err(Error::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Path is not a regular file",
        )));
    }
    
    // Check file extension for Windows executables
    if let Some(extension) = path.extension() {
        let ext = extension.to_string_lossy().to_lowercase();
        if ext != "exe" {
            add_app_log(format!("WARNING: File does not have .exe extension: {}", ext));
        }
    } else {
        add_app_log("WARNING: File has no extension".to_string());
    }
    
    add_app_log("Windows binary permissions check completed".to_string());
    Ok(())
}

pub fn diagnose_binary() -> Result<String> {
    add_app_log("Starting RustFS binary diagnosis...".to_string());
    let binary_path = get_binary_path()?;

    check_permissions(&binary_path)?;

    add_app_log(format!(
        "Testing binary with --help: {}",
        binary_path.display()
    ));
    let output = Command::new(&binary_path)
        .arg("--help")
        .output()
        .map_err(Error::BinaryExecution)?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    add_app_log(format!(
        "Binary --help stdout (first 200 chars): {}",
        stdout.chars().take(200).collect::<String>()
    ));

    if output.status.success() {
        Ok("RustFS binary appears to be working".to_string())
    } else {
        Err(Error::BinaryFailed(output.status.to_string()))
    }
}

pub fn launch(config: RustFsConfig) -> Result<String> {
    add_app_log("Launch command received".to_string());
    add_app_log(format!(
        "Config: data_path={}, port={:?}, host={:?}",
        config.data_path, config.port, config.host
    ));

    if config.data_path.is_empty() {
        return Err(Error::DataPathRequired);
    }

    let binary_path = match &config.binary_path {
        Some(path) => PathBuf::from(path),
        None => get_binary_path()?,
    };
    check_permissions(&binary_path)?;

    // Create logs directory parallel to data_path
    let data_path = Path::new(&config.data_path);
    let logs_dir = if let Some(parent) = data_path.parent() {
        parent.join("logs")
    } else {
        Path::new("logs").to_path_buf()
    };
    add_app_log(format!(
        "Creating logs directory at: {}",
        logs_dir.display()
    ));
    std::fs::create_dir_all(&logs_dir).map_err(Error::Io)?;

    let mut cmd = Command::new(&binary_path);
    cmd.env(
        "RUSTFS_OBS_LOG_DIRECTORY",
        logs_dir.to_string_lossy().to_string(),
    );
    cmd.arg(&config.data_path);

    let address = format!(
        "{}:{}",
        config.host.as_deref().unwrap_or("127.0.0.1"),
        config.port.unwrap_or(9000)
    );
    cmd.arg("--address").arg(&address);

    if let Some(access_key) = &config.access_key {
        cmd.arg("--access-key").arg(access_key);
    }
    if let Some(secret_key) = &config.secret_key {
        cmd.arg("--secret-key").arg(secret_key);
    }
    if config.console_enable {
        cmd.arg("--console-enable");
    }

    add_app_log(format!("Spawning command: {:?}", cmd));
    let mut child = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(Error::BinaryExecution)?;

    let pid = child.id();
    add_app_log(format!("RustFS launched successfully with PID: {}", pid));
    add_rustfs_log("RustFS process started, capturing output...".to_string());

    if let Some(stdout) = child.stdout.take() {
        thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines().filter_map(|l| l.ok()) {
                if line.is_empty() {
                    continue;
                }
                add_rustfs_log(format!("[STDOUT] {}", line));
            }
        });
    }

    if let Some(stderr) = child.stderr.take() {
        thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines().filter_map(|l| l.ok()) {
                if line.is_empty() {
                    continue;
                }
                add_rustfs_log(format!("[STDERR] {}", line));
            }
        });
    }

    // Register the process for tracking before spawning the wait thread
    set_rustfs_process(child);

    Ok(format!("RustFS launched with PID: {}", pid))
}
