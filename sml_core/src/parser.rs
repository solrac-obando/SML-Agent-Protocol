use nom::{
    bytes::complete::{tag, take_until},
    error::Error,
};

#[derive(Debug, Clone, PartialEq)]
pub struct SmlCommand<'a> {
    pub tool: &'a str,
    pub args: Vec<&'a str>,
}

#[derive(Debug)]
enum SmlError {
    MissingStart,
    MissingEnd,
    MissingTool,
    InvalidFormat,
    NomError(String),
}

impl From<nom::Err<Error<&str>>> for SmlError {
    fn from(e: nom::Err<Error<&str>>) -> Self {
        SmlError::NomError(e.to_string())
    }
}

pub fn parse_sml_token<'a>(input: &'a str) -> Option<SmlCommand<'a>> {
    if input.len() < 4 {
        return None;
    }

    let bytes = input.as_bytes();
    
    if bytes[0] != b'@' || bytes[1] != b'[' {
        return None;
    }

    let end_idx = match memchr::memrchr(b']', bytes) {
        Some(idx) if idx == bytes.len() - 1 => idx,
        _ => return None,
    };

    let content = &input[2..end_idx];

    let (tool, args) = match content.find(':') {
        Some(colon_idx) => {
            let tool = &content[..colon_idx];
            let args_raw = &content[colon_idx + 1..];
            
            if tool.is_empty() {
                return None;
            }

            let args: Vec<&str> = if args_raw.is_empty() {
                Vec::new()
            } else {
                args_raw.split('|').collect()
            };

            (tool, args)
        }
        None => {
            let tool = content;
            if tool.is_empty() {
                return None;
            }
            (tool, Vec::new())
        }
    };

    Some(SmlCommand { tool, args })
}

pub fn parse_sml_nom(input: &str) -> Result<SmlCommand<'_>, SmlError> {
    let (remaining, _) = tag("@[")(input)?;
    let (content, _) = take_until("]")(remaining)?;
    let (_, _) = tag("]")(content)?;

    let end = input.len() - 1;
    let content_str = &input[2..end];

    let (tool_part, args_part) = match content_str.find(':') {
        Some(idx) => (&content_str[..idx], &content_str[idx + 1..]),
        None => (content_str, ""),
    };

    if tool_part.is_empty() {
        return Err(SmlError::MissingTool);
    }

    let args: Vec<&str> = if args_part.is_empty() {
        Vec::new()
    } else {
        args_part.split('|').collect()
    };

    Ok(SmlCommand {
        tool: tool_part,
        args,
    })
}

pub fn is_valid_sml(input: &str) -> bool {
    parse_sml_token(input).is_some()
}

pub fn extract_tool(input: &str) -> Option<&str> {
    parse_sml_token(input).map(|cmd| cmd.tool)
}

/// Extrae todos los comandos SML de un texto libre.
///
/// Busca patrones `@[...]` en el texto usando `memchr` y parsea cada uno
/// con `parse_sml_token`. Solo devuelve los comandos válidos.
pub fn extract_sml_commands(text: &str) -> Vec<SmlCommand<'_>> {
    let mut commands = Vec::new();
    let bytes = text.as_bytes();
    let mut pos = 0;

    while pos < bytes.len().saturating_sub(3) {
        match memchr::memchr(b'@', &bytes[pos..]) {
            Some(offset) => {
                let start = pos + offset;
                if start + 1 < bytes.len() && bytes[start + 1] == b'[' {
                    match memchr::memchr(b']', &bytes[start + 2..]) {
                        Some(end_offset) => {
                            let end = start + 2 + end_offset + 1;
                            let candidate = &text[start..end];
                            if let Some(cmd) = parse_sml_token(candidate) {
                                commands.push(cmd);
                            }
                            pos = end;
                        }
                        None => { pos = start + 1; }
                    }
                } else {
                    pos = start + 1;
                }
            }
            None => break,
        }
    }

    commands
}

/// Extrae los comandos SML como strings sin parsear.
/// Útil para logging y re-envío al dispatcher.
pub fn extract_sml_raw(text: &str) -> Vec<&str> {
    let bytes = text.as_bytes();
    let mut results = Vec::new();
    let mut pos = 0;

    while pos < bytes.len().saturating_sub(3) {
        match memchr::memchr(b'@', &bytes[pos..]) {
            Some(offset) => {
                let start = pos + offset;
                if start + 1 < bytes.len() && bytes[start + 1] == b'[' {
                    match memchr::memchr(b']', &bytes[start + 2..]) {
                        Some(end_offset) => {
                            let end = start + 2 + end_offset + 1;
                            let candidate = &text[start..end];
                            if parse_sml_token(candidate).is_some() {
                                results.push(candidate);
                            }
                            pos = end;
                        }
                        None => { pos = start + 1; }
                    }
                } else {
                    pos = start + 1;
                }
            }
            None => break,
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic() {
        let cmd = parse_sml_token("@[read:src/main.rs]").unwrap();
        assert_eq!(cmd.tool, "read");
        assert_eq!(cmd.args, vec!["src/main.rs"]);
    }

    #[test]
    fn test_parse_multiple_args() {
        let cmd = parse_sml_token("@[write:app.py|print('hello')]").unwrap();
        assert_eq!(cmd.tool, "write");
        assert_eq!(cmd.args, vec!["app.py", "print('hello')"]);
    }

    #[test]
    fn test_parse_no_args() {
        let cmd = parse_sml_token("@[term:cargo build]").unwrap();
        assert_eq!(cmd.tool, "term");
        assert_eq!(cmd.args, vec!["cargo build"]);
    }

    #[test]
    fn test_parse_empty_args() {
        let cmd = parse_sml_token("@[read:]").unwrap();
        assert_eq!(cmd.tool, "read");
        assert!(cmd.args.is_empty());
    }

    #[test]
    fn test_invalid_no_start() {
        assert!(parse_sml_token("read:src/main.rs]").is_none());
    }

    #[test]
    fn test_invalid_no_end() {
        assert!(parse_sml_token("@[read:src/main.rs").is_none());
    }

    #[test]
    fn test_invalid_empty_tool() {
        assert!(parse_sml_token("@[::args]").is_none());
    }

    #[test]
    fn test_is_valid_sml() {
        assert!(is_valid_sml("@[read:file.rs]"));
        assert!(!is_valid_sml("not valid"));
    }

    #[test]
    fn test_parse_deep_nested_path() {
        let cmd = parse_sml_token("@[read:src/very/deep/nested/path/to/file.rs]").unwrap();
        assert_eq!(cmd.tool, "read");
        assert_eq!(cmd.args.len(), 1);
    }

    #[test]
    fn test_extract_tool() {
        assert_eq!(extract_tool("@[read:file.rs]"), Some("read"));
        assert_eq!(extract_tool("@[write:file.txt|content]"), Some("write"));
        assert_eq!(extract_tool("no command"), None);
    }

    #[test]
    fn test_extract_sml_commands_from_text() {
        let text = "Voy a leer @[read:file.rs] y luego @[list:src]";
        let cmds = extract_sml_commands(text);
        assert_eq!(cmds.len(), 2);
        assert_eq!(cmds[0].tool, "read");
        assert_eq!(cmds[0].args, vec!["file.rs"]);
        assert_eq!(cmds[1].tool, "list");
        assert_eq!(cmds[1].args, vec!["src"]);
    }

    #[test]
    fn test_extract_sml_commands_no_commands() {
        let text = "Este texto no tiene comandos SML";
        let cmds = extract_sml_commands(text);
        assert!(cmds.is_empty());
    }

    #[test]
    fn test_extract_sml_commands_multiple() {
        let text = "@[read:a.rs] texto @[write:b.rs|hello] más @[term:ls]";
        let cmds = extract_sml_commands(text);
        assert_eq!(cmds.len(), 3);
        assert_eq!(cmds[0].tool, "read");
        assert_eq!(cmds[1].tool, "write");
        assert_eq!(cmds[2].tool, "term");
    }

    #[test]
    fn test_extract_sml_raw() {
        let text = "Hola @[read:file.rs] mundo @[list:.]";
        let raw = extract_sml_raw(text);
        assert_eq!(raw.len(), 2);
        assert_eq!(raw[0], "@[read:file.rs]");
        assert_eq!(raw[1], "@[list:.]");
    }

    #[test]
    fn test_extract_sml_commands_malformed() {
        let text = "Primero @[read:a.rs] y luego @[list:src]";
        let cmds = extract_sml_commands(text);
        assert_eq!(cmds.len(), 2);
        assert_eq!(cmds[0].tool, "read");
        assert_eq!(cmds[1].tool, "list");
    }
}