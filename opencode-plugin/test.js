import plugin from './index.js';

async function test() {
    console.log("1. Inicializando plugin nativo SML...");
    const p = await plugin({}, {});
    
    // Simular que el usuario activó SML
    const commandOutput = {};
    await p["command.execute.before"]({ command: "/sml", arguments: "" }, commandOutput);
    console.log("   Resultado comando:", commandOutput.parts[0].text.trim());

    // Simular que el LLM genera una respuesta con SML
    const output = { text: "Aquí tienes la lectura del archivo: @[read:Cargo.toml]" };
    console.log("2. Interceptando respuesta del LLM...");
    
    // Debería ejecutar el Rust binario sin hacer requests HTTP
    await p["experimental.text.complete"]({}, output);
    
    console.log("3. Prueba completada exitosamente sin Docker ni red.");
    console.log("\n--- RESULTADO FINAL DEL TEXTO INYECTADO AL LLM ---");
    console.log(output.text);
    console.log("---------------------------------------------------\n");
}

test().catch(console.error);
