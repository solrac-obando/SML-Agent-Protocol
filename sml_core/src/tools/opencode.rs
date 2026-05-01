use crate::executor::ExecutorError;
use std::path::Path;
use tokio::fs;
use tokio::process::Command;

pub async fn read_file(path: &str) -> Result<String, ExecutorError> {
    match fs::read_to_string(path).await {
        Ok(content) => Ok(content),
        Err(e) => match e.kind() {
            std::io::ErrorKind::NotFound => Err(ExecutorError::FileNotFound(path.to_string())),
            std::io::ErrorKind::PermissionDenied => Err(ExecutorError::PermissionDenied(path.to_string())),
            _ => Err(ExecutorError::IoError(e.to_string())),
        },
    }
}

pub async fn write_file(path: &str, content: &str) -> Result<String, ExecutorError> {
    let p = Path::new(path);
    
    if let Some(parent) = p.parent() {
        if !parent.exists() {
            if let Err(e) = fs::create_dir_all(parent).await {
                return Err(ExecutorError::IoError(format!("Failed to create directory: {}", e)));
            }
        }
    }

    match fs::write(path, content).await {
        Ok(_) => Ok(format!("[WRITE_OK] {} ({} bytes)", path, content.len())),
        Err(e) => match e.kind() {
            std::io::ErrorKind::PermissionDenied => Err(ExecutorError::PermissionDenied(path.to_string())),
            _ => Err(ExecutorError::IoError(e.to_string())),
        },
    }
}

pub async fn append_file(path: &str, content: &str) -> Result<String, ExecutorError> {
    let p = Path::new(path);
    
    if let Some(parent) = p.parent() {
        if !parent.exists() {
            if let Err(e) = fs::create_dir_all(parent).await {
                return Err(ExecutorError::IoError(format!("Failed to create directory: {}", e)));
            }
        }
    }

    let mut options = tokio::fs::OpenOptions::new();
    options.create(true).append(true);
    
    match options.open(path).await {
        Ok(mut file) => {
            use tokio::io::AsyncWriteExt;
            match file.write_all(content.as_bytes()).await {
                Ok(_) => Ok(format!("[APPEND_OK] {} ({} bytes appended)", path, content.len())),
                Err(e) => Err(ExecutorError::IoError(e.to_string())),
            }
        }
        Err(e) => match e.kind() {
            std::io::ErrorKind::PermissionDenied => Err(ExecutorError::PermissionDenied(path.to_string())),
            _ => Err(ExecutorError::IoError(e.to_string())),
        },
    }
}

pub async fn run_terminal(cmd: &str) -> Result<String, ExecutorError> {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    
    if parts.is_empty() {
        return Err(ExecutorError::IoError("Empty command".to_string()));
    }

    let program = parts[0];
    let args = &parts[1..];

    let output = Command::new(program)
        .args(args)
        .output()
        .await
        .map_err(|e| ExecutorError::IoError(e.to_string()))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if output.status.success() {
        if stdout.is_empty() {
            Ok(format!("[TERM_OK] Exit code: 0"))
        } else {
            Ok(stdout.to_string())
        }
    } else {
        let exit_code = output.status.code().unwrap_or(-1);
        if stderr.is_empty() {
            Ok(format!("[TERM_ERR] Exit code: {}", exit_code))
        } else {
            Ok(format!("[TERM_ERR] {}\n{}", exit_code, stderr))
        }
    }
}

pub async fn list_dir(path: &str) -> Result<String, ExecutorError> {
    let mut entries = Vec::new();
    
    let mut read_dir = fs::read_dir(path).await
        .map_err(|e| ExecutorError::IoError(e.to_string()))?;

    loop {
        match read_dir.next_entry().await {
            Ok(Some(entry)) => {
                let file_name = entry.file_name().to_string_lossy().to_string();
                match entry.file_type().await {
                    Ok(ft) if ft.is_dir() => entries.push(format!("{}/", file_name)),
                    Ok(_) => entries.push(file_name),
                    Err(_) => entries.push(file_name),
                }
            }
            Ok(None) => break,
            Err(_) => break,
        }
    }

    Ok(entries.join("\n"))
}

pub async fn file_exists(path: &str) -> Result<String, ExecutorError> {
    match fs::metadata(path).await {
        Ok(_) => Ok("[EXISTS] true".to_string()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok("[EXISTS] false".to_string()),
        Err(e) => Err(ExecutorError::IoError(e.to_string())),
    }
}

pub async fn file_info(path: &str) -> Result<String, ExecutorError> {
    match fs::metadata(path).await {
        Ok(meta) => {
            let file_type = if meta.is_dir() {
                "directory"
            } else if meta.is_file() {
                "file"
            } else {
                "unknown"
            };
            
            Ok(format!(
                "[INFO] type={} size={} bytes",
                file_type,
                meta.len()
            ))
        }
        Err(e) => Err(ExecutorError::IoError(e.to_string())),
    }
}

pub async fn delete_file(path: &str) -> Result<String, ExecutorError> {
    match fs::remove_file(path).await {
        Ok(_) => Ok(format!("[DELETE_OK] {}", path)),
        Err(e) => Err(ExecutorError::IoError(e.to_string())),
    }
}

pub async fn create_dir(path: &str) -> Result<String, ExecutorError> {
    match fs::create_dir_all(path).await {
        Ok(_) => Ok(format!("[DIR_OK] {}", path)),
        Err(e) => Err(ExecutorError::IoError(e.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[tokio::test]
    async fn test_read_nonexistent() {
        let result = read_file("/nonexistent/path/file.rs").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_run_terminal() {
        let result = run_terminal("echo hello").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_write_and_read_file() {
        let test_path = "test_temp_file.txt";
        let test_content = "Hello, SML!";
        
        let write_result = write_file(test_path, test_content).await;
        assert!(write_result.is_ok());
        
        let read_result = read_file(test_path).await;
        assert!(read_result.is_ok());
        assert!(read_result.unwrap().contains("Hello, SML!"));
        
        let _ = delete_file(test_path).await;
    }

    #[tokio::test]
    async fn test_run_terminal_with_args() {
        let result = run_terminal("ls -la").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_list_current_dir() {
        let result = list_dir(".").await;
        assert!(result.is_ok());
        let content = result.unwrap();
        assert!(!content.is_empty());
    }

    #[tokio::test]
    async fn test_file_exists_true() {
        let result = file_exists("Cargo.toml").await;
        assert!(result.is_ok());
        assert!(result.unwrap().contains("true"));
    }

    #[tokio::test]
    async fn test_file_exists_false() {
        let result = file_exists("/definitely/nonexistent/file.xyz").await;
        assert!(result.is_ok());
        assert!(result.unwrap().contains("false"));
    }

    #[tokio::test]
    async fn test_file_info() {
        let result = file_info("Cargo.toml").await;
        assert!(result.is_ok());
        let info = result.unwrap();
        assert!(info.contains("type=") || info.contains("size="));
    }

    #[tokio::test]
    async fn test_create_and_delete_dir() {
        let test_dir = "test_temp_dir";
        
        let create_result = create_dir(test_dir).await;
        assert!(create_result.is_ok());
        
        let exist_result = file_exists(test_dir).await;
        assert!(exist_result.is_ok());
        assert!(exist_result.unwrap().contains("true"));
        
        let _ = delete_file(test_dir).await;
    }

    #[tokio::test]
    async fn test_run_terminal_failed_command() {
        let result = run_terminal("false").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_write_creates_parent_dirs() {
        let test_path = "deep/nested/dir/test_file.txt";
        let test_content = "nested content";
        
        let result = write_file(test_path, test_content).await;
        assert!(result.is_ok());
        
        let _ = delete_file(test_path).await;
        let _ = delete_file("deep/nested/dir").await;
        let _ = delete_file("deep/nested").await;
        let _ = delete_file("deep").await;
    }
}