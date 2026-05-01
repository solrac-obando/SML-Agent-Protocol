use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct OllamaClient {
    client: Client,
    base_url: String,
    model: String,
    system_prompt: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
    options: ChatOptions,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatOptions {
    temperature: f32,
    num_ctx: usize,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    message: ChatMessage,
    done: bool,
}

impl OllamaClient {
    pub fn new(model: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: "http://localhost:11434".to_string(),
            model: model.to_string(),
            system_prompt: Self::default_system_prompt(),
        }
    }

    fn default_system_prompt() -> String {
        r#"Eres un agente con acceso a herramientas del sistema.

Usa SOLO comandos SML (Symbolic Micro-Language) para interactuar con archivos y terminal:

HERRAMIENTAS DISPONIBLES:
- @[read:ruta] - Lee un archivo
- @[write:ruta|contenido] - Escribe un archivo
- @[term:comando] - Ejecuta comando en terminal
- @[list:ruta] - Lista directorio
- @[exist:ruta] - Verifica si existe
- @[info:ruta] - Información de archivo
- @[mkdir:ruta] - Crea directorio
- @[delete:ruta] - Elimina archivo

REGLAS:
1. Cuando necesites leer/escribir/ejecutar algo, responde SOLO con el comando SML
2. El usuario puede ejecutar el comando por ti
3. No inventes comandos - usa solo los listados arriba
4. Para operaciones complejas, hazlo paso a paso

EJEMPLO:
Pregunta: ¿Qué hay en el archivo config.json?
Respuesta: @[read:config.json]"#.to_string()
    }

    pub fn with_system_prompt(mut self, prompt: &str) -> Self {
        self.system_prompt = prompt.to_string();
        self
    }

    pub fn system_prompt(&self) -> &str {
        &self.system_prompt
    }

    pub async fn chat(&self, messages: Vec<ChatMessage>) -> Result<String, ClientError> {
        let request = ChatRequest {
            model: self.model.clone(),
            messages,
            stream: false,
            options: ChatOptions {
                temperature: 0.3,
                num_ctx: 4096,
            },
        };

        let response = self
            .client
            .post(format!("{}/api/chat", self.base_url))
            .json(&request)
            .send()
            .await
            .map_err(ClientError::Network)?
            .json::<ChatResponse>()
            .await
            .map_err(ClientError::Parse)?;

        Ok(response.message.content)
    }

    pub async fn chat_with_tools(&self, messages: Vec<ChatMessage>) -> Result<(String, Vec<String>), ClientError> {
        let response = self.chat(messages).await?;
        
        let sml_regex = regex_lite::Regex::new(r"@\[([a-z_]+):([^\]]+)\]").map_err(ClientError::Regex)?;
        let commands: Vec<String> = sml_regex
            .find_iter(&response)
            .map(|m| m.as_str().to_string())
            .collect();

        Ok((response, commands))
    }
}

#[derive(Debug)]
pub enum ClientError {
    Network(reqwest::Error),
    Parse(reqwest::Error),
    Regex(regex_lite::Error),
}

impl std::fmt::Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientError::Network(e) => write!(f, "[NETWORK_ERROR] {}", e),
            ClientError::Parse(e) => write!(f, "[PARSE_ERROR] {}", e),
            ClientError::Regex(e) => write!(f, "[REGEX_ERROR] {}", e),
        }
    }
}

impl std::error::Error for ClientError {}

impl From<reqwest::Error> for ClientError {
    fn from(e: reqwest::Error) -> Self {
        ClientError::Network(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_prompt() {
        let client = OllamaClient::new("qwen2.5-coder:3b");
        assert!(!client.system_prompt.is_empty());
        assert!(client.system_prompt.contains("@[read:"));
    }

    #[test]
    fn test_custom_system_prompt() {
        let client = OllamaClient::new("qwen2.5-coder:3b").with_system_prompt("Custom prompt");
        assert_eq!(client.system_prompt, "Custom prompt");
    }
}