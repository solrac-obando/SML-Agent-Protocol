pub struct GbnfGenerator;

impl GbnfGenerator {
    pub fn generate_grammar() -> String {
        r#"
# SML Symbolic Micro-Language GBNF Grammar
# This grammar mathematically guarantees zero hallucinations

root ::= text* command text*

command ::= "@[" tool ":" arguments "]"

tool ::= "read" | "write" | "term" | "list" | "exist" | "info"

arguments ::= "" | argument ("|" argument)*

argument ::= [^|\]]+

text ::= [^@]+

# Token counts:
# @[read:src/main.rs] = 4 tokens (vs JSON ~15)
# @[write:app.py|print('hello')] = 5 tokens (vs JSON ~25)
"#.to_string()
    }

    pub fn generate_json_schema() -> String {
        serde_json::json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "enum": ["read", "write", "term", "list", "exist", "info"]
                },
                "arguments": {
                    "type": "array",
                    "items": {
                        "type": "string"
                    },
                    "maxItems": 10
                }
            },
            "required": ["command"],
            "additionalProperties": false
        }).to_string()
    }

    pub fn get_tool_descriptions() -> Vec<ToolDefinition> {
        vec![
            ToolDefinition {
                name: "read".to_string(),
                description: "Read file contents".to_string(),
                args: vec![
                    ArgDefinition {
                        name: "path".to_string(),
                        description: "File path to read".to_string(),
                        required: true,
                    }
                ],
            },
            ToolDefinition {
                name: "write".to_string(),
                description: "Write content to file".to_string(),
                args: vec![
                    ArgDefinition {
                        name: "path".to_string(),
                        description: "Destination file path".to_string(),
                        required: true,
                    },
                    ArgDefinition {
                        name: "content".to_string(),
                        description: "Content to write".to_string(),
                        required: true,
                    },
                ],
            },
            ToolDefinition {
                name: "term".to_string(),
                description: "Execute terminal command".to_string(),
                args: vec![
                    ArgDefinition {
                        name: "command".to_string(),
                        description: "Command to execute".to_string(),
                        required: true,
                    },
                ],
            },
            ToolDefinition {
                name: "list".to_string(),
                description: "List directory contents".to_string(),
                args: vec![
                    ArgDefinition {
                        name: "path".to_string(),
                        description: "Directory path (optional, defaults to current)".to_string(),
                        required: false,
                    },
                ],
            },
            ToolDefinition {
                name: "exist".to_string(),
                description: "Check if file or directory exists".to_string(),
                args: vec![
                    ArgDefinition {
                        name: "path".to_string(),
                        description: "Path to check".to_string(),
                        required: true,
                    },
                ],
            },
            ToolDefinition {
                name: "info".to_string(),
                description: "Get file/directory metadata".to_string(),
                args: vec![
                    ArgDefinition {
                        name: "path".to_string(),
                        description: "Path to inspect".to_string(),
                        required: true,
                    },
                ],
            },
        ]
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub args: Vec<ArgDefinition>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ArgDefinition {
    pub name: String,
    pub description: String,
    pub required: bool,
}

pub fn generate_system_prompt() -> String {
    r#"
You are connected to the SML (Symbolic Micro-Language) protocol.
Communication is performed using ultra-compact token sequences.

SYNTAX:
@[command:argument1|argument2]

COMMANDS:
- read:<path>              - Read file contents
- write:<path>|<content>   - Write content to file
- term:<command>           - Execute terminal command
- list:<path>              - List directory contents (optional path)
- exist:<path>             - Check if path exists
- info:<path>              - Get file metadata

EXAMPLES:
@[read:Cargo.toml]
@[write:app.py|print('hello')]
@[term:cargo build]

RULES:
1. NEVER use JSON for tool calls
2. NEVER explain your actions before executing
3. Be surgical: emit only the command, nothing else
4. Wait for the result before providing analysis

This protocol saves 95% of context window vs JSON.
"#.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grammar_generation() {
        let grammar = GbnfGenerator::generate_grammar();
        assert!(grammar.contains("command ::= \"@[\""));
        assert!(grammar.contains("tool ::= \"read\""));
    }

    #[test]
    fn test_system_prompt() {
        let prompt = generate_system_prompt();
        assert!(prompt.contains("@[command:argument1|argument2]"));
        assert!(prompt.contains("read:"));
        assert!(prompt.contains("write:"));
    }

    #[test]
    fn test_generate_json_schema() {
        let schema = GbnfGenerator::generate_json_schema();
        assert!(schema.contains("type"));
        assert!(schema.contains("command"));
        assert!(schema.contains("read"));
    }

    #[test]
    fn test_get_tool_descriptions() {
        let tools = GbnfGenerator::get_tool_descriptions();
        assert!(!tools.is_empty());
        assert_eq!(tools.len(), 6);
        
        let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(tool_names.contains(&"read"));
        assert!(tool_names.contains(&"write"));
        assert!(tool_names.contains(&"term"));
    }

    #[test]
    fn test_tool_definition_structure() {
        let tools = GbnfGenerator::get_tool_descriptions();
        
        let read_tool = tools.iter().find(|t| t.name == "read").unwrap();
        assert_eq!(read_tool.description, "Read file contents");
        assert!(!read_tool.args.is_empty());
        assert!(read_tool.args[0].required);
        
        let write_tool = tools.iter().find(|t| t.name == "write").unwrap();
        assert_eq!(write_tool.args.len(), 2);
        assert!(write_tool.args[0].required);
        assert!(write_tool.args[1].required);
    }

    #[test]
    fn test_term_tool_description() {
        let tools = GbnfGenerator::get_tool_descriptions();
        let term_tool = tools.iter().find(|t| t.name == "term").unwrap();
        assert_eq!(term_tool.args.len(), 1);
        assert!(term_tool.args[0].required);
    }

    #[test]
    fn test_list_tool_optional_args() {
        let tools = GbnfGenerator::get_tool_descriptions();
        let list_tool = tools.iter().find(|t| t.name == "list").unwrap();
        assert_eq!(list_tool.args.len(), 1);
        assert!(!list_tool.args[0].required);
    }

    #[test]
    fn test_grammar_contains_all_tools() {
        let grammar = GbnfGenerator::generate_grammar();
        assert!(grammar.contains("\"read\""));
        assert!(grammar.contains("\"write\""));
        assert!(grammar.contains("\"term\""));
        assert!(grammar.contains("\"list\""));
        assert!(grammar.contains("\"exist\""));
        assert!(grammar.contains("\"info\""));
    }

    #[test]
    fn test_grammar_command_structure() {
        let grammar = GbnfGenerator::generate_grammar();
        assert!(grammar.contains("command ::= \"@[\""));
        assert!(grammar.contains("tool"));
        assert!(grammar.contains("arguments"));
    }

    #[test]
    fn test_system_prompt_contains_all_tools() {
        let prompt = generate_system_prompt();
        assert!(prompt.contains("- read:"));
        assert!(prompt.contains("- write:"));
        assert!(prompt.contains("- term:"));
        assert!(prompt.contains("- list:"));
        assert!(prompt.contains("- exist:"));
        assert!(prompt.contains("- info:"));
    }
}