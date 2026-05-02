mod parser;
mod executor;
mod tools;
mod llm_bridge;
mod stress_tests;
mod ollama_client;
mod http_server;

use parser::{parse_sml_token, SmlCommand};
use executor::dispatch;
use ollama_client::OllamaClient;
use std::env;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("SML Dispatcher v0.1.0 - Symbolic Micro-Language Protocol");
    println!("Waiting for SML commands (format: @[command:arg1|arg2])...\n");

    let args: Vec<String> = env::args().collect();
    
    if args.len() > 1 {
        if args[1] == "--benchmark" {
            run_benchmarks().await?;
            return Ok(());
        }
        
        if args[1] == "--test-parser" {
            test_parser()?;
            return Ok(());
        }

        if args.len() > 2 && args[1] == "--execute" {
            let result = dispatch(parse_sml_token(&args[2]).unwrap()).await;
            println!("{}", result);
            return Ok(());
        }

        if args[1] == "--ollama" {
            let model = args.get(2).cloned().unwrap_or_else(|| "qwen2.5-coder:3b".to_string());
            ollama_chat_mode(model.as_str()).await?;
            return Ok(());
        }

        if args[1] == "--server" {
            let port: u16 = args.get(2)
                .and_then(|p| p.parse().ok())
                .unwrap_or(8080);
            http_server::start_server(port).await?;
            return Ok(());
        }
    }

    interactive_mode().await?;

    Ok(())
}

async fn interactive_mode() -> Result<(), Box<dyn std::error::Error>> {
    let (tx, mut rx) = mpsc::channel::<String>(100);

    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin).lines();

    let tx_clone = tx.clone();
    tokio::spawn(async move {
        while let Ok(Some(line)) = reader.next_line().await {
            let _ = tx_clone.send(line).await;
        }
    });

    while let Some(line) = rx.recv().await {
        if let Some(cmd) = parse_sml_token(&line) {
            println!("[SML] Parsed: tool={}, args={:?}", cmd.tool, cmd.args);
            let result = dispatch(cmd).await;
            println!("{}", result);
        } else if line.starts_with("@[") {
            println!("[ERR:INVALID_SYNTAX]");
        }
    }

    Ok(())
}

async fn run_benchmarks() -> Result<(), Box<dyn std::error::Error>> {
    println!("Running micro-benchmarks...\n");
    
    let test_commands = [
        "@[read:src/main.rs]",
        "@[write:app.py|print('hello')]",
        "@[term:cargo build]",
        "@[read:config.json]",
    ];

    use std::time::Instant;

    for cmd in &test_commands {
        let start = Instant::now();
        for _ in 0..10000 {
            let _ = parse_sml_token(cmd);
        }
        let elapsed = start.elapsed();
        println!("{}: {:.2}ns/op", cmd, elapsed.as_nanos() as f64 / 10000.0);
    }

    Ok(())
}

fn test_parser() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing parser...\n");

    let test_cases = [
        ("@[read:src/main.rs]", Some(("read", vec!["src/main.rs"]))),
        ("@[write:app.py|print('hello')]", Some(("write", vec!["app.py", "print('hello')"]))),
        ("@[term:cargo build]", Some(("term", vec!["cargo build"]))),
        ("@[read:]", Some(("read", vec![""]))),
        ("plain text", None),
        ("@[invalid", None),
        ("not a command]", None),
    ];

    for (input, expected) in &test_cases {
        let result = parse_sml_token(input);
        match (result, expected) {
            (Some(cmd), Some((tool, args))) => {
                if cmd.tool == *tool && cmd.args.len() == args.len() {
                    println!("✓ {} -> tool={}", input, cmd.tool);
                } else {
                    println!("✗ {} -> expected ({}, {:?}), got ({}, {:?})", input, tool, args, cmd.tool, cmd.args);
                }
            }
            (None, None) => println!("✓ {} -> correctly rejected", input),
            (Some(cmd), None) => println!("✗ {} -> should be rejected but got {}", input, cmd.tool),
            (None, Some(_)) => println!("✗ {} -> should be accepted but was rejected", input),
        }
    }

    Ok(())
}

async fn ollama_chat_mode(model: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== SML Chat con Ollama ({}) ===", model);
    println!("Escribe tu pregunta (Ctrl+C para salir)\n");

    let client = OllamaClient::new(model);
    let mut messages = vec![
        ollama_client::ChatMessage {
            role: "system".to_string(),
            content: client.system_prompt().to_string(),
        }
    ];

    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin).lines();

    loop {
        print!("\n> ");
        std::io::Write::flush(&mut std::io::stdout())?;

        let line = match reader.next_line().await {
            Ok(Some(l)) => l,
            Ok(None) => break,
            Err(_) => break,
        };

        if line.trim().is_empty() {
            continue;
        }

        messages.push(ollama_client::ChatMessage {
            role: "user".to_string(),
            content: line,
        });

        loop {
            match client.chat_with_tools(messages.clone()).await {
                Ok((response, commands)) => {
                    messages.push(ollama_client::ChatMessage {
                        role: "assistant".to_string(),
                        content: response.clone(),
                    });

                    if response.contains("@") {
                         println!("\n🤖: {}", response);
                    } else if commands.is_empty() {
                         println!("\n🤖: {}", response);
                    }

                    if !commands.is_empty() {
                        println!("\n📟 Ejecutando herramientas:");
                        let mut results = String::new();
                        for cmd in &commands {
                            print!("  {}", cmd);
                            
                            // Seguridad: Pedir confirmación para borrar
                            if cmd.starts_with("@[delete:") {
                                print!(" [!] ¿Confirmar borrado? (y/N): ");
                                std::io::Write::flush(&mut std::io::stdout())?;
                                let mut input = String::new();
                                std::io::stdin().read_line(&mut input)?;
                                if input.trim().to_lowercase() != "y" {
                                    println!(" → CANCELADO");
                                    results.push_str(&format!("\nResultado de {}: [ERR:USER_DENIED]", cmd));
                                    continue;
                                }
                            }

                            if let Some(parsed) = parse_sml_token(cmd) {
                                let result = dispatch(parsed).await;
                                println!(" → OK");
                                results.push_str(&format!("\nResultado de {}: {}", cmd, result));
                            }
                        }
                        
                        messages.push(ollama_client::ChatMessage {
                            role: "user".to_string(),
                            content: format!("Resultados de ejecución:{}", results),
                        });
                        
                        // Volvemos a llamar a Ollama para que procese los resultados
                        continue;
                    }
                    break;
                }
                Err(e) => {
                    println!("Error: {}", e);
                    break;
                }
            }
        }
    }

    println!("\n¡Hasta luego!");
    Ok(())
}