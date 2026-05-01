use crate::parser::SmlCommand;
use crate::tools::opencode;
use crate::tools::{custom_env, generic_env};
use std::time::{Duration, Instant};

#[derive(Debug)]
pub enum ExecutorError {
    UnknownCommand(String),
    FileNotFound(String),
    PermissionDenied(String),
    IoError(String),
    Timeout,
}

impl std::fmt::Display for ExecutorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutorError::UnknownCommand(cmd) => write!(f, "[ERR:UNKNOWN_CMD:{}]", cmd),
            ExecutorError::FileNotFound(path) => write!(f, "[ERR:FILE_NOT_FOUND:{}]", path),
            ExecutorError::PermissionDenied(path) => write!(f, "[ERR:PERMISSION_DENIED:{}]", path),
            ExecutorError::IoError(msg) => write!(f, "[ERR:IO:{}]", msg),
            ExecutorError::Timeout => write!(f, "[ERR:TIMEOUT]"),
        }
    }
}

impl std::error::Error for ExecutorError {}

pub async fn dispatch(cmd: SmlCommand<'_>) -> String {
    let start = Instant::now();
    let result = dispatch_inner(&cmd).await;
    let elapsed = start.elapsed();

    match result {
        Ok(output) => {
            if elapsed.as_millis() > 100 {
                format!("[OK:{}ms] {}", elapsed.as_millis(), output)
            } else {
                output
            }
        }
        Err(e) => {
            format!("{}", e)
        }
    }
}

async fn dispatch_inner(cmd: &SmlCommand<'_>) -> Result<String, ExecutorError> {
    match cmd.tool {
        "read" => {
            if cmd.args.is_empty() {
                return Err(ExecutorError::UnknownCommand("read requires path argument".to_string()));
            }
            opencode::read_file(cmd.args[0]).await
        }
        "write" => {
            if cmd.args.len() < 2 {
                return Err(ExecutorError::UnknownCommand("write requires path and content".to_string()));
            }
            opencode::write_file(cmd.args[0], cmd.args[1]).await
        }
        "append" => {
            if cmd.args.len() < 2 {
                return Err(ExecutorError::UnknownCommand("append requires path and content".to_string()));
            }
            opencode::append_file(cmd.args[0], cmd.args[1]).await
        }
        "term" => {
            if cmd.args.is_empty() {
                return Err(ExecutorError::UnknownCommand("term requires command argument".to_string()));
            }
            opencode::run_terminal(cmd.args[0]).await
        }
        "list" => {
            if cmd.args.is_empty() {
                opencode::list_dir(".").await
            } else {
                opencode::list_dir(cmd.args[0]).await
            }
        }
        "exist" => {
            if cmd.args.is_empty() {
                return Err(ExecutorError::UnknownCommand("exist requires path argument".to_string()));
            }
            opencode::file_exists(cmd.args[0]).await
        }
        "info" => {
            if cmd.args.is_empty() {
                return Err(ExecutorError::UnknownCommand("info requires path argument".to_string()));
            }
            opencode::file_info(cmd.args[0]).await
        }
        "sublime" => {
            if cmd.args.is_empty() || cmd.args[0] != "open" || cmd.args.len() < 2 {
                return Err(ExecutorError::UnknownCommand("sublime:open requires path".to_string()));
            }
            Ok(custom_env::sublime_open(cmd.args[1]))
        }
        "vscode" => {
            if cmd.args.is_empty() || cmd.args[0] != "open" || cmd.args.len() < 2 {
                return Err(ExecutorError::UnknownCommand("vscode:open requires path".to_string()));
            }
            Ok(custom_env::vscode_open(cmd.args[1]))
        }
        "python" => {
            if cmd.args.is_empty() || cmd.args[0] != "run" || cmd.args.len() < 2 {
                return Err(ExecutorError::UnknownCommand("python:run requires script path".to_string()));
            }
            Ok(custom_env::python_run(cmd.args[1]))
        }
        "browser" => {
            if cmd.args.is_empty() {
                return Err(ExecutorError::UnknownCommand("browser requires action".to_string()));
            }
            match cmd.args[0] {
                "search" => {
                    if cmd.args.len() < 2 {
                        return Err(ExecutorError::UnknownCommand("browser:search requires query".to_string()));
                    }
                    Ok(custom_env::browser_search(cmd.args[1]))
                }
                "open" => {
                    if cmd.args.len() < 2 {
                        return Err(ExecutorError::UnknownCommand("browser:open requires url".to_string()));
                    }
                    Ok(generic_env::browser_open(cmd.args[1]))
                }
                _ => Err(ExecutorError::UnknownCommand(format!("browser:{}", cmd.args[0])))
            }
        }
        "office" => {
            if cmd.args.is_empty() || cmd.args[0] != "writer" {
                return Err(ExecutorError::UnknownCommand("office:writer requires valid action".to_string()));
            }
            let path = if cmd.args.len() < 2 { "" } else { cmd.args[1] };
            Ok(custom_env::libreoffice_writer(path))
        }
        "editor" => {
            if cmd.args.is_empty() || cmd.args[0] != "open" || cmd.args.len() < 2 {
                return Err(ExecutorError::UnknownCommand("editor:open requires path".to_string()));
            }
            Ok(generic_env::editor_open(cmd.args[1]))
        }
        _ => Err(ExecutorError::UnknownCommand(cmd.tool.to_string())),
    }
}

pub async fn dispatch_with_timeout(cmd: SmlCommand<'_>, timeout: Duration) -> Result<String, ExecutorError> {
    match tokio::time::timeout(timeout, dispatch(cmd)).await {
        Ok(result) => {
            if result.starts_with("[ERR:") {
                Err(ExecutorError::IoError(result))
            } else {
                Ok(result)
            }
        }
        Err(_) => Err(ExecutorError::Timeout),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_dispatch_read() {
        let cmd = SmlCommand {
            tool: "read",
            args: vec!["Cargo.toml"],
        };
        let result = dispatch(cmd).await;
        assert!(result.contains("[OK") || result.contains("name = \"sml_core\""));
    }

    #[tokio::test]
    async fn test_dispatch_unknown() {
        let cmd = SmlCommand {
            tool: "nonexistent",
            args: vec![],
        };
        let result = dispatch(cmd).await;
        assert!(result.contains("[ERR:UNKNOWN_CMD"));
    }

    #[tokio::test]
    async fn test_dispatch_write() {
        let cmd = SmlCommand {
            tool: "write",
            args: vec!["test_output.txt", "test content"],
        };
        let result = dispatch(cmd).await;
        assert!(result.contains("[WRITE_OK]") || result.contains("test_output.txt"));
    }

    #[tokio::test]
    async fn test_dispatch_term() {
        let cmd = SmlCommand {
            tool: "term",
            args: vec!["echo hello"],
        };
        let result = dispatch(cmd).await;
        assert!(result.contains("hello") || result.contains("[TERM_OK]"));
    }

    #[tokio::test]
    async fn test_dispatch_list() {
        let cmd = SmlCommand {
            tool: "list",
            args: vec!["."],
        };
        let result = dispatch(cmd).await;
        assert!(!result.is_empty());
    }

    #[tokio::test]
    async fn test_dispatch_exist() {
        let cmd = SmlCommand {
            tool: "exist",
            args: vec!["Cargo.toml"],
        };
        let result = dispatch(cmd).await;
        assert!(result.contains("[EXISTS] true") || result.contains("true"));
    }

    #[tokio::test]
    async fn test_dispatch_info() {
        let cmd = SmlCommand {
            tool: "info",
            args: vec!["Cargo.toml"],
        };
        let result = dispatch(cmd).await;
        assert!(result.contains("[INFO]") || result.contains("type=") || result.contains("size="));
    }

    #[tokio::test]
    async fn test_dispatch_read_no_args() {
        let cmd = SmlCommand {
            tool: "read",
            args: vec![],
        };
        let result = dispatch(cmd).await;
        assert!(result.contains("[ERR:UNKNOWN_CMD"));
    }

    #[tokio::test]
    async fn test_dispatch_write_no_args() {
        let cmd = SmlCommand {
            tool: "write",
            args: vec!["only_path"],
        };
        let result = dispatch(cmd).await;
        assert!(result.contains("[ERR:UNKNOWN_CMD"));
    }

    #[tokio::test]
    async fn test_error_display() {
        let err = ExecutorError::FileNotFound("/test/path".to_string());
        assert!(format!("{}", err).contains("/test/path"));
        assert!(format!("{}", err).contains("[ERR:FILE_NOT_FOUND"));
    }
}