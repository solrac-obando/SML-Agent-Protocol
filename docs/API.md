# API Reference - SML Protocol

## Funciones Principales

### Parser (`parser.rs`)

#### `parse_sml_token(input: &str) -> Option<SmlCommand>`

Parsea un comando SML desde una cadena de texto.

**Parametros:**
- `input`: Cadena con formato `@[tool:args]`

**Retorna:**
- `Some(SmlCommand)` si el parseo es exitoso
- `None` si el formato es inválido

**Ejemplo:**
```rust
let cmd = parse_sml_token("@[read:src/main.rs]");
assert_eq!(cmd.unwrap().tool, "read");
```

---

#### `is_valid_sml(input: &str) -> bool`

Verifica si una cadena tiene formato SML válido.

---

#### `extract_tool(input: &str) -> Option<&str>`

Extrae solo el nombre de la herramienta del comando SML.

---

### Executor (`executor.rs`)

#### `dispatch(cmd: SmlCommand<'_>) -> String`

Ejecuta un comando SML y retorna el resultado.

**Comandos soportados:**
- `read` - Lee un archivo
- `write` - Escribe un archivo
- `term` - Ejecuta comando terminal
- `list` - Lista directorio
- `exist` - Verifica existencia
- `info` - Obtiene metadata

---

#### `dispatch_with_timeout(cmd: SmlCommand<'_>, timeout: Duration) -> Result<String, ExecutorError>`

Igual que `dispatch` pero con timeout.

---

### Herramientas (`tools/opencode.rs`)

#### `read_file(path: &str) -> Result<String, ExecutorError>`

Lee el contenido de un archivo.

#### `write_file(path: &str, content: &str) -> Result<String, ExecutorError>`

Escribe contenido a un archivo.

#### `run_terminal(cmd: &str) -> Result<String, ExecutorError>`

Ejecuta un comando en la terminal.

#### `list_dir(path: &str) -> Result<String, ExecutorError>`

Lista el contenido de un directorio.

#### `file_exists(path: &str) -> Result<String, ExecutorError>`

Verifica si un archivo/directorio existe.

#### `file_info(path: &str) -> Result<String, ExecutorError>`

Obtiene información de archivo (tamaño, tipo).

---

## Estructuras de Datos

### `SmlCommand<'a>`

```rust
pub struct SmlCommand<'a> {
    pub tool: &'a str,       // Nombre de la herramienta
    pub args: Vec<&'a str>,  // Argumentos
}
```

### `ExecutorError`

```rust
pub enum ExecutorError {
    UnknownCommand(String),
    FileNotFound(String),
    PermissionDenied(String),
    IoError(String),
    Timeout,
}
```

---

## Uso desde CLI

```bash
# Ejecutar comando directo
sml --execute "@[read:Cargo.toml]"

# Modo benchmark
sml --benchmark

# Ver ayuda
sml --help
```