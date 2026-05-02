/**
 * OpenCode Plugin para Integración SML
 * Intercepta comandos y modifica el prompt del sistema para usar SML (Symbolic Micro-Language)
 * Despacha comandos al servidor HTTP nativo de Rust en localhost:8080
 */

const SML_SERVER_URL = 'http://127.0.0.1:8080';
let smlEnabled = false;
let serverAvailable = false;

/**
 * Verificar si el servidor SML está corriendo
 */
async function checkServerHealth() {
    try {
        const res = await fetch(`${SML_SERVER_URL}/health`, {
            signal: AbortSignal.timeout(2000)
        });
        if (res.ok) {
            const data = await res.json();
            serverAvailable = true;
            return data;
        }
    } catch (_) {
        serverAvailable = false;
    }
    return null;
}

/**
 * Ejecutar un comando SML vía HTTP al dispatcher Rust
 */
async function executeSmlCommand(cmd) {
    try {
        const res = await fetch(`${SML_SERVER_URL}/execute`, {
            method: 'POST',
            headers: { 'Content-Type': 'text/plain' },
            body: cmd,
            signal: AbortSignal.timeout(30000)
        });
        if (res.ok) {
            return await res.text();
        }
        return `[ERR:HTTP_${res.status}] ${await res.text()}`;
    } catch (err) {
        return `[ERR:CONNECTION] ${err.message}`;
    }
}

export default async function plugin(input, options) {
    return {
        // Interceptamos comandos del usuario antes de que se envíen al LLM
        "command.execute.before": async (args, output) => {
            if (args.command.trim() === "/sml") {
                smlEnabled = !smlEnabled;

                // Verificar salud del servidor al activar
                let statusMsg = '';
                if (smlEnabled) {
                    const health = await checkServerHealth();
                    if (health) {
                        statusMsg = `\n  ✅ Servidor SML conectado (v${health.version})`;
                    } else {
                        statusMsg = '\n  ⚠️  Servidor SML no detectado. Ejecuta: sml_core --server 8080';
                    }
                }

                output.parts = [
                    {
                        type: "text",
                        text: `\n[⚙️ OpenCode-SML] Comunicación SML ${smlEnabled ? "ACTIVADA ✅" : "DESACTIVADA ❌"}${statusMsg}\n`
                    }
                ];

                args.command = "";
                args.arguments = "";
            }
        },

        // Modificamos el System Prompt del LLM de forma dinámica si SML está activado
        "experimental.chat.system.transform": async (args, output) => {
            if (smlEnabled) {
                output.system.push(
                    "============================================================",
                    "CRITICAL INSTRUCTION: You MUST use the Symbolic Micro-Language (SML) instead of standard JSON or Markdown for tool calls.",
                    "SML Syntax: @[tool_name:arg1|arg2|arg3]",
                    "Available tools:",
                    "- read: @[read:path/to/file]",
                    "- write: @[write:path/to/file|content]",
                    "- term: @[term:command]",
                    "- list: @[list:path]",
                    "- exist: @[exist:path]",
                    "- info: @[info:path]",
                    "- mkdir: @[mkdir:path]",
                    "- delete: @[delete:path] (REQUIRES USER CONFIRMATION)",
                    "Never use JSON to call these tools. Output exactly the SML syntax.",
                    "============================================================"
                );
            }
        },

        // Hook para interceptar las respuestas del LLM que contengan SML
        // y despacharlas al servidor HTTP de Rust
        "experimental.text.complete": async (args, output) => {
            if (smlEnabled && output.text.includes("@[")) {
                // Re-verificar servidor si no estaba disponible
                if (!serverAvailable) {
                    await checkServerHealth();
                }

                if (!serverAvailable) {
                    output.text += '\n\n> ⚠️ **SML Server no disponible.** Ejecuta `sml_core --server 8080`';
                    return;
                }

                try {
                    // Extraer todos los comandos SML del texto de respuesta
                    const smlCommands = output.text.match(/@\[.*?\]/g);

                    if (smlCommands) {
                        for (const cmd of smlCommands) {
                            const result = await executeSmlCommand(cmd);

                            // Inyectamos el resultado directamente en la respuesta
                            output.text += `\n\n**Resultado SML (\`${cmd}\`):**\n\`\`\`\n${result.trim()}\n\`\`\``;
                        }
                    }
                } catch (error) {
                    console.error("[SML Plugin Error]:", error);
                    output.text += `\n\n> ❌ **Error SML:** ${error.message}`;
                }
            }
        }
    };
}
