/**
 * OpenCode Plugin para Integración SML
 * Intercepta comandos y modifica el prompt del sistema para usar SML (Symbolic Micro-Language)
 */
import { execFile } from 'child_process';
import { promisify } from 'util';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const execFileAsync = promisify(execFile);

export default async function plugin(input, options) {
    // SML siempre activado por defecto en este entorno
    const smlEnabled = true;

    return {

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
                    "- append: @[append:path/to/file|content]",
                    "- term: @[term:command]",
                    "- list: @[list:path]",
                    "- exist: @[exist:path]",
                    "- info: @[info:path]",
                    "- sublime: @[sublime:open|path/to/file]",
                    "- vscode: @[vscode:open|path/to/file]",
                    "- python: @[python:run|path/to/script.py]",
                    "- browser search: @[browser:search|query]",
                    "- browser open: @[browser:open|url]",
                    "- libreoffice: @[office:writer|path/to/file]",
                    "- generic editor: @[editor:open|path/to/file]",
                    "Never use JSON to call these tools. Output exactly the SML syntax.",
                    "============================================================"
                );
            }
        },

        // Hook para interceptar las respuestas del texto del LLM que contengan SML 
        // y despacharlas de forma nativa a nuestro binario compilado localmente
        "experimental.text.complete": async (args, output) => {
            if (smlEnabled && output.text.includes("@[")) {
                try {
                    // Extraer todos los comandos SML del texto de respuesta
                    const smlCommands = output.text.match(/@\[.*?\]/g);
                    
                    if (smlCommands) {
                        for (const cmd of smlCommands) {
                            // Ejecutar binario nativo compilado de Rust
                            // Resolviendo ruta absoluta segura
                            const binaryPath = path.resolve(__dirname, '../sml_core/target/release/sml_core');
                            
                            try {
                                const { stdout, stderr } = await execFileAsync(binaryPath, ['--execute', cmd]);
                                if (stderr) {
                                    console.warn(`[SML Native Warning]: ${stderr}`);
                                    output.text += `\n\n[SML Warning]: ${stderr}`;
                                }
                                
                                // Inyectamos la respuesta SML directamente de vuelta al LLM / Chat
                                output.text += `\n\n**Resultado SML (${cmd}):**\n\`\`\`\n${stdout.trim()}\n\`\`\``;
                            } catch (err) {
                                console.error(`[SML Native Execution Error]: ${err.message}`);
                            }
                        }
                    }
                } catch (error) {
                    console.error("[SML Plugin Error]:", error);
                }
            }
        }
    };
}
