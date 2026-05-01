# SML Developer Guide / Guía del Desarrollador / Guia do Desenvolvedor

[English](#english) | [Español](#español) | [Português](#português)

---

<a name="english"></a>
## 🇬🇧 English: How to Create Custom SML Commands

SML (Symbolic Micro-Language) is designed to be highly extensible. If you want to connect a new AI model to your local environment (e.g., to edit photos, control smart home devices, or interact with specific APIs), you can easily create new commands in the Rust core.

### Step-by-Step Guide

#### 1. Define the Command Syntax
Decide on your command's structure: `@[tool:action|arg1|arg2]`. 
*Example: `@[photo:edit|image.png|grayscale]`*

#### 2. Create the Rust Tool Logic
Create a new file or add a function in `sml_core/src/tools/`.
For a generic/OS-agnostic tool, edit `generic_env.rs`. For a custom local tool, edit `custom_env.rs`.

```rust
// sml_core/src/tools/generic_env.rs
use std::process::Command;
use crate::tools::sandbox::is_safe_path;

pub fn photo_edit(path: &str, effect: &str) -> String {
    if !is_safe_path(path) {
        return "[ERR:SECURITY_SANDBOX_BLOCKED]".to_string();
    }
    // Example: Call a Python script or an CLI image editor
    match Command::new("convert").arg(path).arg(format!("-{}", effect)).arg(path).output() {
        Ok(_) => format!("[OK:PHOTO_EDITED] {}", path),
        Err(e) => format!("[ERR:PHOTO_FAILED] {}", e),
    }
}
```

#### 3. Route the Command in Executor
Open `sml_core/src/executor.rs` and add your new command to the `dispatch_inner` match statement. The `cmd.tool` corresponds to the text before the colon (`photo`), and `cmd.args[0]` corresponds to the action (`edit`).

```rust
// sml_core/src/executor.rs
        "photo" => {
            if cmd.args.is_empty() || cmd.args[0] != "edit" || cmd.args.len() < 3 {
                return Err(ExecutorError::UnknownCommand("photo:edit requires path and effect".to_string()));
            }
            Ok(generic_env::photo_edit(cmd.args[1], cmd.args[2]))
        }
```

#### 4. Update the LLM System Prompt
If you are using the OpenCode plugin, open `opencode-plugin/index.js` and add your new command to the `Available tools` list so the AI knows it exists:
```javascript
"- photo edit: @[photo:edit|path/to/image|effect]",
```

#### 5. Compile and Test
Compile the Rust core to apply changes:
```bash
cd sml_core
cargo build --release
```
Test it via CLI:
```bash
./target/release/sml_core --execute "@[photo:edit|test.png|grayscale]"
```

---

<a name="español"></a>
## 🇪🇸 Español: Cómo Crear Comandos SML Personalizados

SML es altamente extensible. Si deseas que tu IA controle herramientas nuevas (edición de fotos, APIs locales, etc.), puedes crear nuevos comandos directamente en el núcleo de Rust.

### Guía Paso a Paso

#### 1. Define la Sintaxis del Comando
Decide la estructura. Formato: `@[herramienta:accion|arg1|arg2]`. 
*Ejemplo: `@[photo:edit|imagen.png|grayscale]`*

#### 2. Crea la Lógica en Rust
Crea una función en `sml_core/src/tools/`.
Si es una herramienta para cualquier SO, usa `generic_env.rs`. Si es específica para tu computadora (ej. Ubuntu), usa `custom_env.rs`.

```rust
// sml_core/src/tools/generic_env.rs
use std::process::Command;
use crate::tools::sandbox::is_safe_path;

pub fn photo_edit(path: &str, effect: &str) -> String {
    if !is_safe_path(path) {
        return "[ERR:SECURITY_SANDBOX_BLOCKED]".to_string();
    }
    match Command::new("convert").arg(path).arg(format!("-{}", effect)).arg(path).output() {
        Ok(_) => format!("[OK:PHOTO_EDITED] {}", path),
        Err(e) => format!("[ERR:PHOTO_FAILED] {}", e),
    }
}
```

#### 3. Enruta el Comando en el Ejecutor
Abre `sml_core/src/executor.rs` y añade tu comando al bloque `match` en `dispatch_inner`.

```rust
// sml_core/src/executor.rs
        "photo" => {
            if cmd.args.is_empty() || cmd.args[0] != "edit" || cmd.args.len() < 3 {
                return Err(ExecutorError::UnknownCommand("photo:edit requiere ruta y efecto".to_string()));
            }
            Ok(generic_env::photo_edit(cmd.args[1], cmd.args[2]))
        }
```

#### 4. Actualiza el System Prompt
Abre `opencode-plugin/index.js` y añade tu herramienta a la lista para que la IA sepa usarla:
```javascript
"- photo edit: @[photo:edit|path/to/image|effect]",
```

#### 5. Compila y Prueba
```bash
cd sml_core
cargo build --release
./target/release/sml_core --execute "@[photo:edit|test.png|grayscale]"
```

---

<a name="português"></a>
## 🇧🇷 Português: Como Criar Comandos SML Personalizados

O SML é altamente extensível. Se você deseja que sua IA controle novas ferramentas (edição de fotos, APIs locais, etc.), pode criar novos comandos diretamente no núcleo em Rust.

### Guia Passo a Passo

#### 1. Defina a Sintaxe do Comando
Decida a estrutura. Formato: `@[ferramenta:acao|arg1|arg2]`. 
*Exemplo: `@[photo:edit|imagem.png|grayscale]`*

#### 2. Crie a Lógica em Rust
Crie uma função em `sml_core/src/tools/`.
Se for uma ferramenta para qualquer sistema operacional, use `generic_env.rs`.

```rust
// sml_core/src/tools/generic_env.rs
use std::process::Command;
use crate::tools::sandbox::is_safe_path;

pub fn photo_edit(path: &str, effect: &str) -> String {
    if !is_safe_path(path) {
        return "[ERR:SECURITY_SANDBOX_BLOCKED]".to_string();
    }
    match Command::new("convert").arg(path).arg(format!("-{}", effect)).arg(path).output() {
        Ok(_) => format!("[OK:PHOTO_EDITED] {}", path),
        Err(e) => format!("[ERR:PHOTO_FAILED] {}", e),
    }
}
```

#### 3. Roteie o Comando no Executor
Abra `sml_core/src/executor.rs` e adicione seu comando ao bloco `match` em `dispatch_inner`.

```rust
// sml_core/src/executor.rs
        "photo" => {
            if cmd.args.is_empty() || cmd.args[0] != "edit" || cmd.args.len() < 3 {
                return Err(ExecutorError::UnknownCommand("photo:edit requer caminho e efeito".to_string()));
            }
            Ok(generic_env::photo_edit(cmd.args[1], cmd.args[2]))
        }
```

#### 4. Atualize o System Prompt
Abra `opencode-plugin/index.js` e adicione sua ferramenta à lista:
```javascript
"- photo edit: @[photo:edit|path/to/image|effect]",
```

#### 5. Compile e Teste
```bash
cd sml_core
cargo build --release
./target/release/sml_core --execute "@[photo:edit|test.png|grayscale]"
```
