# Arquitectura Técnica - SML Protocol

## Visión General

```
┌─────────────────────────────────────────────────────────────┐
│                     LLM (Ollama/llama.cpp)                  │
│                                                             │
│  Genera: @[command:arg1|arg2]  (SML)                       │
│  vs                                                     │
│  {"tool":"read","parameters":{...}} (JSON)                 │
└─────────────────────┬───────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────┐
│                    SML Core (Rust)                         │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
│  │   Parser    │→ │  Executor   │→ │   Tools     │        │
│  │ Zero-Alloc  │  │   Async     │  │  FS/Term   │        │
│  └─────────────┘  └─────────────┘  └─────────────┘        │
└─────────────────────────────────────────────────────────────┘
```

## Componentes

### 1. Parser (Zero-Allocation)

**Ubicación**: `src/parser.rs`

El parser está diseñado para:
- No realizar asignaciones en el heap
- Usar lifetimes para referenciar la memoria original
- Parsear en tiempo O(1)

```rust
pub fn parse_sml_token<'a>(input: &'a str) -> Option<SmlCommand<'a>> {
    if input.starts_with("@[") && input.ends_with(']') {
        // Operamos directamente sobre las referencias
        let content = &input[2..input.len() - 1];
        // ... parseo sin copias
    }
    None
}
```

### 2. Executor (Async)

**Ubicación**: `src/executor.rs`

Dispatch de comandos usando Tokio para operaciones async:
- No bloquea el thread principal
- Manejo de errores estructurado
- Timeout opcional

### 3. Tools (FileSystem/Terminal)

**Ubicación**: `src/tools/opencode.rs`

Implementa las operaciones reales:
- `read_file`: Lee archivos del sistema
- `write_file`: Escribe archivos
- `run_terminal`: Ejecuta comandos shell
- `list_dir`: Lista directorios
- `file_exists` / `file_info`: Consultas metadata

### 4. LLM Bridge

**Ubicación**: `src/llm_bridge/`

- `gbnf.rs`: Generador de gramática GBNF
- `ffi.rs`: Placeholder para integración llama.cpp

---

## Flujo de Ejecución

1. **LLM genera respuesta** → Formato SML (`@[tool:args]`)
2. **Parser recibe cadena** → Extrae tool y args
3. **Executor recibe comando** → Routing al tool apropiado
4. **Tool ejecuta** → Operación del sistema de archivos/terminal
5. **Resultado retorna** → Se inyecta al contexto del LLM

---

## Decisiones de Diseño

### Zero-Allocation

El parser NO copia strings. Solo crea referencias (slices) a la memoria donde vive la salida del LLM.

### O(1) Parsing

El parsing usa búsqueda simple de caracteres (`starts_with`, `ends_with`, `find`) sin iteraciones complejas.

### Async I/O

Todas las operaciones de archivos y terminal son no-bloqueantes usando Tokio.

---

## Testing

- **Unit tests**: 53 tests en módulos individuales
- **Stress tests**: 10 tests de rendimiento
- **Integration**: Tests con Ollama live