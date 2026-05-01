# SML - Symbolic Micro-Language Protocol

[English](#english) | [Español](#español) | [Português](#português)

---
<a name="english"></a>
## 🇬🇧 SML (Symbolic Micro-Language) Protocol

**Version**: 1.0.0 | **Status**: Production

SML is an ultra-efficient protocol designed to replace traditional JSON/MCP schemas in the communication between local LLMs and agentic environments like OpenCode.

### Key Features
- **Token Reduction**: 50-95% vs JSON.
- **Parse Time**: O(1) Zero-allocation parsing.
- **Native Execution**: Processor-level speed via Rust.
- **Extensible**: Add agentic tools (Python execution, VS Code, photo editing) in minutes.

### Quick Start
```bash
cd sml_core
cargo build --release
# Single execution
./target/release/sml_core --execute "@[read:Cargo.toml]"
# Interactive autonomous chat with Ollama
./target/release/sml_core --ollama qwen2.5-coder:3b
```

### Supported SML Commands
- **File System**: `@[read:path]`, `@[write:path|content]`, `@[append:path|content]`, `@[list:dir]`, `@[exist:path]`, `@[info:path]`
- **System / Execute**: `@[term:command]`, `@[python:run|script.py]`
- **IDE / Editors**: `@[sublime:open|path]`, `@[vscode:open|path]`, `@[editor:open|path]`
- **Apps**: `@[browser:open|url]`, `@[browser:search|query]`, `@[office:writer|path]`

### Documentation
- [Developer Guide (How to add commands)](docs/DEVELOPER_GUIDE.md)
- [API Reference](docs/API.md)
- [Performance Benchmarks](docs/PERFORMANCE.md)

---
<a name="español"></a>
## 🇪🇸 Protocolo SML (Symbolic Micro-Language)

**Versión**: 1.0.0 | **Estado**: Producción

SML es un protocolo ultra-eficiente diseñado para reemplazar los esquemas JSON/MCP tradicionales en la comunicación entre LLMs locales y entornos agenticos como OpenCode.

### Características Clave
- **Reducción de tokens**: 50-95% vs JSON.
- **Tiempo de parsing**: O(1) Zero-allocation.
- **Ejecución nativa**: Velocidad de procesador vía Rust.
- **Extensible**: Añade herramientas agenticas (Python, VS Code, etc.) en minutos.

### Inicio Rápido
```bash
cd sml_core
cargo build --release
# Ejecución única
./target/release/sml_core --execute "@[read:Cargo.toml]"
# Chat autónomo interactivo con Ollama
./target/release/sml_core --ollama qwen2.5-coder:3b
```

### Comandos SML Soportados
- **Archivos**: `@[read:ruta]`, `@[write:ruta|contenido]`, `@[append:ruta|contenido]`, `@[list:dir]`, `@[exist:ruta]`, `@[info:ruta]`
- **Sistema / Ejecución**: `@[term:comando]`, `@[python:run|script.py]`
- **IDEs**: `@[sublime:open|ruta]`, `@[vscode:open|ruta]`, `@[editor:open|ruta]`
- **Apps**: `@[browser:open|url]`, `@[browser:search|query]`, `@[office:writer|ruta]`

### Documentación
- [Guía del Desarrollador (Cómo crear comandos)](docs/DEVELOPER_GUIDE.md)
- [Referencia de API](docs/API.md)
- [Métricas de Rendimiento](docs/PERFORMANCE.md)

---
<a name="português"></a>
## 🇧🇷 Protocolo SML (Symbolic Micro-Language)

**Versão**: 1.0.0 | **Status**: Produção

SML é um protocolo ultra-eficiente projetado para substituir os esquemas tradicionais JSON/MCP na comunicação entre LLMs locais e ambientes de agentes como o OpenCode.

### Recursos Principais
- **Redução de tokens**: 50-95% em comparação com JSON.
- **Tempo de parsing**: O(1) Zero-allocation.
- **Execução nativa**: Velocidade no nível do processador via Rust.
- **Extensível**: Adicione ferramentas de agentes (Python, VS Code, etc.) em minutos.

### Início Rápido
```bash
cd sml_core
cargo build --release
# Execução única
./target/release/sml_core --execute "@[read:Cargo.toml]"
# Chat autônomo interativo com Ollama
./target/release/sml_core --ollama qwen2.5-coder:3b
```

### Comandos SML Suportados
- **Arquivos**: `@[read:caminho]`, `@[write:caminho|conteudo]`, `@[append:caminho|conteudo]`, `@[list:dir]`, `@[exist:caminho]`, `@[info:caminho]`
- **Sistema / Execução**: `@[term:comando]`, `@[python:run|script.py]`
- **IDEs**: `@[sublime:open|caminho]`, `@[vscode:open|caminho]`, `@[editor:open|caminho]`
- **Apps**: `@[browser:open|url]`, `@[browser:search|query]`, `@[office:writer|caminho]`

### Documentação
- [Guia do Desenvolvedor (Como criar comandos)](docs/DEVELOPER_GUIDE.md)
- [Referência da API](docs/API.md)
- [Desempenho](docs/PERFORMANCE.md)

---
*MIT License*