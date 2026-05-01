# SML OpenCode Plugin

Este es el plugin de integración oficial para **OpenCode** que permite al entorno agéntico (ya sea con un modelo local o en la nube) utilizar el protocolo SML en lugar de llamadas a herramientas pesadas (JSON/MCP).

## Instalación en OpenCode

1. Abre la configuración de tu entorno OpenCode (generalmente `~/.opencode/package.json` o configuración de plugins).
2. Agrega este directorio a la lista de plugins.

```json
{
  "plugin": [
    "/home/carlosobando/proyectos_IA/microlenguaje-IA-instintivo/opencode-plugin"
  ]
}
```

## Uso

Dentro de tu terminal interactiva de OpenCode, simplemente escribe el siguiente comando:

```bash
/sml
```

### ¿Qué hace `/sml`?
1. Funciona como un "interruptor" (toggle). Al activarlo, el plugin inserta dinámicamente un prompt de sistema estricto (System Prompt) que prohíbe al LLM enviar respuestas JSON o llamar herramientas nativas.
2. Obliga a que el modelo (local o de nube) devuelva la sintaxis ultraligera de SML: `@[herramienta:argumento]`.
3. Intercepta el texto plano que genera el LLM, busca el comando de SML, y lo redirige internamente vía API hacia el contenedor Docker (`sml-dispatcher` corriendo en el puerto 8080).

¡De esta manera tu comunicación está **100% integrada** y no necesitas sustituir OpenCode, sino que le añades la capa optimizada SML!
