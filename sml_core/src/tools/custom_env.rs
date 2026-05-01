use std::process::Command;
use crate::tools::sandbox::is_safe_path;

pub fn sublime_open(path: &str) -> String {
    if !is_safe_path(path) {
        return "[ERR:SECURITY_SANDBOX_BLOCKED]".to_string();
    }
    match Command::new("/snap/bin/subl").arg(path).spawn() {
        Ok(_) => format!("[OK:SUBLIME_OPENED] {}", path),
        Err(e) => format!("[ERR:SUBLIME_FAILED] {}", e),
    }
}

pub fn browser_search(query: &str) -> String {
    let safe_query = query.replace(' ', "+");
    let url = format!("https://google.com/search?q={}", safe_query);
    match Command::new("/usr/bin/google-chrome").arg(&url).spawn() {
        Ok(_) => format!("[OK:BROWSER_SEARCH] {}", query),
        Err(e) => format!("[ERR:BROWSER_FAILED] {}", e),
    }
}

pub fn libreoffice_writer(path: &str) -> String {
    // Si la ruta no está vacía y no es segura, bloqueamos
    if !path.is_empty() && !is_safe_path(path) {
        return "[ERR:SECURITY_SANDBOX_BLOCKED]".to_string();
    }
    
    let mut cmd = Command::new("/usr/bin/libreoffice");
    cmd.arg("--writer");
    
    if !path.is_empty() {
        cmd.arg(path);
    }
    
    match cmd.spawn() {
        Ok(_) => format!("[OK:LIBREOFFICE_WRITER] {}", path),
        Err(e) => format!("[ERR:LIBREOFFICE_FAILED] {}", e),
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
    match Command::new("/usr/bin/python3").arg(path).output() {
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
