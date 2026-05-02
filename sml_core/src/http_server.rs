use crate::parser::{parse_sml_token, extract_sml_raw};
use crate::executor::dispatch;
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::time::Instant;

const VERSION: &str = "0.1.0";

/// Inicia el servidor HTTP SML en el puerto especificado.
///
/// Endpoints:
/// - GET  /health   → {"status":"ok","version":"0.1.0"}
/// - POST /execute  → Ejecuta un comando SML (body: texto plano)
/// - POST /batch    → Ejecuta múltiples comandos separados por \n
pub async fn start_server(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await?;

    println!("╔══════════════════════════════════════════════╗");
    println!("║  SML HTTP Server v{}                     ║", VERSION);
    println!("║  Escuchando en: http://{}           ║", addr);
    println!("║                                              ║");
    println!("║  Endpoints:                                  ║");
    println!("║    GET  /health  → Health check              ║");
    println!("║    POST /execute → Ejecutar comando SML      ║");
    println!("║    POST /batch   → Ejecutar batch de SML     ║");
    println!("╚══════════════════════════════════════════════╝\n");

    loop {
        let (mut stream, peer_addr) = listener.accept().await?;

        tokio::spawn(async move {
            let start = Instant::now();
            let mut buf = vec![0u8; 65536]; // 64KB buffer

            let n = match stream.read(&mut buf).await {
                Ok(0) => return,
                Ok(n) => n,
                Err(_) => return,
            };

            let request_str = String::from_utf8_lossy(&buf[..n]);

            // Parse HTTP request line
            let first_line = request_str.lines().next().unwrap_or("");
            let parts: Vec<&str> = first_line.split_whitespace().collect();

            if parts.len() < 2 {
                let _ = send_response(&mut stream, 400, "text/plain", "Bad Request").await;
                return;
            }

            let method = parts[0];
            let path = parts[1];

            // Extraer body (después de \r\n\r\n)
            let body = request_str
                .find("\r\n\r\n")
                .map(|i| &request_str[i + 4..])
                .unwrap_or("")
                .trim();

            let elapsed_label = format!("[{}] {} {} from {}", 
                chrono_now(), method, path, peer_addr);

            let response = match (method, path) {
                ("GET", "/health") => {
                    let json = format!(
                        r#"{{"status":"ok","version":"{}","uptime_check":true}}"#,
                        VERSION
                    );
                    send_response(&mut stream, 200, "application/json", &json).await
                }

                ("POST", "/execute") => {
                    if body.is_empty() {
                        send_response(&mut stream, 400, "text/plain", 
                            "[ERR:EMPTY_BODY] Envía un comando SML en el body").await
                    } else {
                        let result = execute_sml_command(body).await;
                        send_response(&mut stream, 200, "text/plain", &result).await
                    }
                }

                ("POST", "/batch") => {
                    if body.is_empty() {
                        send_response(&mut stream, 400, "application/json",
                            r#"{"error":"empty body"}"#).await
                    } else {
                        let result = execute_batch(body).await;
                        send_response(&mut stream, 200, "application/json", &result).await
                    }
                }

                ("OPTIONS", _) => {
                    // Preflight CORS
                    send_response(&mut stream, 204, "text/plain", "").await
                }

                _ => {
                    let msg = format!(
                        r#"{{"error":"not_found","message":"Ruta no encontrada: {} {}","available":["/health","/execute","/batch"]}}"#,
                        method, path
                    );
                    send_response(&mut stream, 404, "application/json", &msg).await
                }
            };

            let elapsed = start.elapsed();
            println!("{} → {:.2}ms", elapsed_label, elapsed.as_secs_f64() * 1000.0);

            if let Err(e) = response {
                eprintln!("  Error enviando respuesta: {}", e);
            }
        });
    }
}

/// Ejecuta un solo comando SML desde texto (puede ser un comando directo o texto con un @[...])
async fn execute_sml_command(input: &str) -> String {
    // Primero intentar parsear como comando directo
    if let Some(cmd) = parse_sml_token(input) {
        return dispatch(cmd).await;
    }

    // Si no es un comando directo, buscar comandos embebidos en el texto
    let raw_commands = extract_sml_raw(input);
    if raw_commands.is_empty() {
        return "[ERR:NO_SML_COMMAND] No se encontró ningún comando SML válido".to_string();
    }

    // Ejecutar el primer comando encontrado
    if let Some(cmd) = parse_sml_token(raw_commands[0]) {
        dispatch(cmd).await
    } else {
        "[ERR:PARSE_FAILED] No se pudo parsear el comando".to_string()
    }
}

/// Ejecuta múltiples comandos SML separados por newline
async fn execute_batch(input: &str) -> String {
    let mut results = Vec::new();

    // Extraer comandos: cada línea puede ser un comando, o buscar @[...] en el texto completo
    let raw_commands = extract_sml_raw(input);

    if raw_commands.is_empty() {
        // Intentar línea por línea
        for line in input.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Some(cmd) = parse_sml_token(trimmed) {
                let result = dispatch(cmd).await;
                results.push(format!(
                    r#"{{"command":"{}","result":"{}","status":"ok"}}"#,
                    escape_json(trimmed),
                    escape_json(&result)
                ));
            }
        }
    } else {
        for raw in &raw_commands {
            if let Some(cmd) = parse_sml_token(raw) {
                let result = dispatch(cmd).await;
                results.push(format!(
                    r#"{{"command":"{}","result":"{}","status":"ok"}}"#,
                    escape_json(raw),
                    escape_json(&result)
                ));
            }
        }
    }

    format!("[{}]", results.join(","))
}

/// Envía una respuesta HTTP con headers CORS
async fn send_response(
    stream: &mut tokio::net::TcpStream,
    status: u16,
    content_type: &str,
    body: &str,
) -> Result<(), std::io::Error> {
    let status_text = match status {
        200 => "OK",
        204 => "No Content",
        400 => "Bad Request",
        404 => "Not Found",
        500 => "Internal Server Error",
        _ => "Unknown",
    };

    let response = format!(
        "HTTP/1.1 {} {}\r\n\
         Content-Type: {}\r\n\
         Content-Length: {}\r\n\
         Access-Control-Allow-Origin: *\r\n\
         Access-Control-Allow-Methods: GET, POST, OPTIONS\r\n\
         Access-Control-Allow-Headers: Content-Type, Authorization\r\n\
         Connection: close\r\n\
         X-SML-Version: {}\r\n\
         \r\n\
         {}",
        status, status_text,
        content_type,
        body.len(),
        VERSION,
        body
    );

    stream.write_all(response.as_bytes()).await?;
    stream.flush().await?;
    Ok(())
}

/// Escape básico para JSON strings
fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// Timestamp simple sin dependencias externas
fn chrono_now() -> String {
    use std::time::SystemTime;
    let duration = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    let hours = (secs / 3600) % 24;
    let minutes = (secs / 60) % 60;
    let seconds = secs % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_json() {
        assert_eq!(escape_json("hello \"world\""), "hello \\\"world\\\"");
        assert_eq!(escape_json("line1\nline2"), "line1\\nline2");
        assert_eq!(escape_json("path\\to\\file"), "path\\\\to\\\\file");
    }

    #[test]
    fn test_chrono_now_format() {
        let ts = chrono_now();
        assert_eq!(ts.len(), 8); // HH:MM:SS
        assert_eq!(&ts[2..3], ":");
        assert_eq!(&ts[5..6], ":");
    }

    #[tokio::test]
    async fn test_execute_sml_command_direct() {
        let result = execute_sml_command("@[exist:Cargo.toml]").await;
        assert!(result.contains("EXISTS"));
    }

    #[tokio::test]
    async fn test_execute_sml_command_embedded() {
        let result = execute_sml_command("por favor ejecuta @[exist:Cargo.toml]").await;
        assert!(result.contains("EXISTS"));
    }

    #[tokio::test]
    async fn test_execute_sml_command_invalid() {
        let result = execute_sml_command("texto sin comandos").await;
        assert!(result.contains("ERR"));
    }

    #[tokio::test]
    async fn test_execute_batch() {
        let input = "@[exist:Cargo.toml]\n@[exist:nonexistent.xyz]";
        let result = execute_batch(input).await;
        assert!(result.starts_with('['));
        assert!(result.ends_with(']'));
        assert!(result.contains("Cargo.toml"));
    }
}
