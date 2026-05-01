use std::process::Command;
use std::env;
use crate::tools::sandbox::is_safe_path;

pub fn editor_open(path: &str) -> String {
    if !is_safe_path(path) {
        return "[ERR:SECURITY_SANDBOX_BLOCKED]".to_string();
    }
    let editor = env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());
    match Command::new(&editor).arg(path).spawn() {
        Ok(_) => format!("[OK:EDITOR_OPENED] {}", path),
        Err(e) => format!("[ERR:EDITOR_FAILED] {}", e),
    }
}

pub fn browser_open(url: &str) -> String {
    let cmd = if cfg!(target_os = "windows") {
        "start"
    } else if cfg!(target_os = "macos") {
        "open"
    } else {
        "xdg-open" // Linux default
    };

    match Command::new(cmd).arg(url).spawn() {
        Ok(_) => format!("[OK:BROWSER_OPENED] {}", url),
        Err(e) => format!("[ERR:BROWSER_FAILED] {}", e),
    }
}

pub fn vscode_open(path: &str) -> String {
    if !path.is_empty() && !is_safe_path(path) {
        return "[ERR:SECURITY_SANDBOX_BLOCKED]".to_string();
    }
    match Command::new("code").arg(path).spawn() {
        Ok(_) => format!("[OK:VSCODE_OPENED] {}", path),
        Err(e) => format!("[ERR:VSCODE_FAILED] {}", e),
    }
}

pub fn python_run(path: &str) -> String {
    if !is_safe_path(path) {
        return "[ERR:SECURITY_SANDBOX_BLOCKED]".to_string();
    }
    
    let python_cmd = if cfg!(target_os = "windows") {
        "python"
    } else {
        "python3"
    };

    match Command::new(python_cmd).arg(path).output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            if output.status.success() {
                format!("[OK:PYTHON_RUN] {}\n{}", path, stdout)
            } else {
                format!("[ERR:PYTHON_FAILED] {}\n{}", output.status.code().unwrap_or(-1), stderr)
            }
        },
        Err(e) => format!("[ERR:PYTHON_SYSTEM_FAILED] {}", e),
    }
}
