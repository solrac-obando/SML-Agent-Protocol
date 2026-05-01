#[cfg(test)]
mod stress_tests {
    use crate::parser::{parse_sml_token, is_valid_sml, SmlCommand};
    use crate::executor::dispatch;
    use std::time::Instant;

    #[tokio::test]
    async fn stress_parse_10k_commands() {
        let commands = [
            "@[read:src/main.rs]",
            "@[write:app.py|print('hello')]",
            "@[term:cargo build]",
            "@[list:src]",
            "@[exist:Cargo.toml]",
            "@[info:src/lib.rs]",
            "@[read:very/long/path/to/file.rs]",
            "@[write:file.txt|content with spaces]",
            "@[term:ls -la | grep test]",
            "@[read:]",
        ];
        
        let start = Instant::now();
        for _ in 0..1000 {
            for cmd in &commands {
                let _ = parse_sml_token(cmd);
            }
        }
        let elapsed = start.elapsed();
        
        println!("Parsed 10,000 commands in {:?}", elapsed);
        assert!(elapsed.as_millis() < 5000);
    }

    #[tokio::test]
    async fn stress_parse_invalid_inputs() {
        let invalid_inputs = [
            "plain text",
            "no brackets",
            "@[incomplete",
            "@no closing",
            "@[::]",
            "@[tool:]",
            "random noise here @[read:file.rs]",
            "",
            "   ",
            "@[",
        ];
        
        let start = Instant::now();
        for _ in 0..1000 {
            for input in &invalid_inputs {
                let _ = parse_sml_token(input);
            }
        }
        let elapsed = start.elapsed();
        
        println!("Parsed 10,000 invalid inputs in {:?}", elapsed);
        assert!(elapsed.as_millis() < 3000);
    }

    #[tokio::test]
    async fn stress_is_valid_sml() {
        let start = Instant::now();
        for _ in 0..10000 {
            assert!(is_valid_sml("@[read:file.rs]"));
            assert!(!is_valid_sml("invalid"));
        }
        let elapsed = start.elapsed();
        
        println!("Validated 20,000 inputs in {:?}", elapsed);
        assert!(elapsed.as_millis() < 2000);
    }

    #[tokio::test]
    async fn stress_dispatch_read_multiple_files() {
        let files = [
            "Cargo.toml",
            "src/main.rs",
            "src/lib.rs",
            "README.md",
            "tests/test.rs",
        ];
        
        let start = Instant::now();
        for _ in 0..100 {
            for file in &files {
                let cmd = SmlCommand {
                    tool: "read",
                    args: vec![*file],
                };
                let _ = dispatch(cmd).await;
            }
        }
        let elapsed = start.elapsed();
        
        println!("Dispatched 500 read commands in {:?}", elapsed);
        assert!(elapsed.as_secs() < 60);
    }

    #[tokio::test]
    async fn stress_dispatch_terminal_commands() {
        let commands = [
            "echo test",
            "pwd",
            "ls",
            "date",
            "whoami",
        ];
        
        let start = Instant::now();
        for _ in 0..50 {
            for cmd_str in &commands {
                let cmd = SmlCommand {
                    tool: "term",
                    args: vec![*cmd_str],
                };
                let _ = dispatch(cmd).await;
            }
        }
        let elapsed = start.elapsed();
        
        println!("Dispatched 250 terminal commands in {:?}", elapsed);
        assert!(elapsed.as_secs() < 120);
    }

    #[tokio::test]
    async fn stress_write_and_read_cycle() {
        let start = Instant::now();
        
        for i in 0..20 {
            let path = format!("stress_test_{}.txt", i);
            let content = format!("Test content number {}", i);
            
            let write_cmd = SmlCommand {
                tool: "write",
                args: vec![path.as_str(), content.as_str()],
            };
            let _ = dispatch(write_cmd).await;
            
            let read_cmd = SmlCommand {
                tool: "read",
                args: vec![path.as_str()],
            };
            let result = dispatch(read_cmd).await;
            assert!(result.contains(&format!("{}", i)));
        }
        
        let elapsed = start.elapsed();
        println!("Completed 20 write-read cycles in {:?}", elapsed);
        assert!(elapsed.as_secs() < 30);
    }

    #[tokio::test]
    async fn stress_list_directory() {
        let start = Instant::now();
        
        for _ in 0..100 {
            let cmd = SmlCommand {
                tool: "list",
                args: vec!["."],
            };
            let _ = dispatch(cmd).await;
        }
        
        let elapsed = start.elapsed();
        println!("Listed directory 100 times in {:?}", elapsed);
        assert!(elapsed.as_secs() < 30);
    }

    #[tokio::test]
    async fn stress_exist_check() {
        let paths = [
            "Cargo.toml",
            "src/main.rs",
            "/nonexistent/file.txt",
            ".",
            "..",
        ];
        
        let start = Instant::now();
        for _ in 0..50 {
            for path in &paths {
                let cmd = SmlCommand {
                    tool: "exist",
                    args: vec![*path],
                };
                let _ = dispatch(cmd).await;
            }
        }
        let elapsed = start.elapsed();
        
        println!("Checked existence 250 times in {:?}", elapsed);
        assert!(elapsed.as_secs() < 30);
    }

    #[tokio::test]
    async fn stress_mixed_operations() {
        let start = Instant::now();
        
        for i in 0..30 {
            let cmd = SmlCommand {
                tool: "read",
                args: vec!["Cargo.toml"],
            };
            let _ = dispatch(cmd).await;
            
            let cmd = SmlCommand {
                tool: "term",
                args: vec!["echo test"],
            };
            let _ = dispatch(cmd).await;
            
            let cmd = SmlCommand {
                tool: "list",
                args: vec!["."],
            };
            let _ = dispatch(cmd).await;
            
            let path = format!("temp_file_{}.txt", i);
            let cmd = SmlCommand {
                tool: "write",
                args: vec![path.as_str(), "test"],
            };
            let _ = dispatch(cmd).await;
            
            let cmd = SmlCommand {
                tool: "exist",
                args: vec![path.as_str()],
            };
            let _ = dispatch(cmd).await;
        }
        
        let elapsed = start.elapsed();
        println!("Completed 150 mixed operations in {:?}", elapsed);
        assert!(elapsed.as_secs() < 60);
    }

    #[tokio::test]
    async fn stress_parser_with_special_chars() {
        let special_commands = [
            "@[read:file with spaces.txt]",
            "@[write:path/to/file.txt|content with | pipes]",
            "@[term:echo 'hello world']",
            "@[read:unicode_파일.txt]",
            "@[write:test.rs|fn main() { println!(\"test\"); }]",
        ];
        
        let start = Instant::now();
        for _ in 0..200 {
            for cmd in &special_commands {
                let _ = parse_sml_token(cmd);
            }
        }
        let elapsed = start.elapsed();
        
        println!("Parsed 1000 special char commands in {:?}", elapsed);
        assert!(elapsed.as_millis() < 3000);
    }
}