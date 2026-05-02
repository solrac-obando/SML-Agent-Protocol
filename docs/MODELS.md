# Modelos Ollama Probados

## Resultados de Tests

### Tests de Eficiencia (Prompt Simple)

| Modelo | Output Tokens | JSON Equiv. | Ahorro |
|--------|----------------|-------------|--------|
| qwen2.5-coder:1.5b | 7 | ~15 | **53%** |
| gemma3:1b | 10 | ~15 | 33% |

### Tests de Tarea Compleja (Multi-step)

| Modelo | Action Tokens | Commands | Context Saved |
|--------|---------------|----------|---------------|
| gemma3:1b | 5-10 | 4-10 | 35-40 tokens |
| qwen2.5-coder:1.5b | 30-35 | 10 | 10-15 tokens |

---

## Modelos Disponibles en el Sistema

| Modelo | Tamaño | Estado | Notas |
|--------|--------|--------|-------|
| gemma4:e2b | 7.2 GB | Disponible | Mayor capacidad |
| deepseek-r1:7b | 4.7 GB | Disponible | Mejor razonamiento |
| gemma3:4b | 3.3 GB | Disponible | Uso general |
| ministral-3:3b | 3.0 GB | Disponible | Buen rendimiento |
| nemotron-3-nano:4b | 2.8 GB | Disponible | Eficiente |
| qwen3:4b | 2.5 GB | Disponible | Rápido |
| cogito:3b | 2.2 GB | Disponible | Razonamiento |
| llama3.2:3b | 2.0 GB | Disponible | Confiable |
| granite3.1-moe:3b | 2.0 GB | Disponible | Código |
| qwen2.5-coder:3b | 1.9 GB | **Verificado** | **Mejor para código** |
| deepseek-r1:1.5b | 1.1 GB | Disponible | Compacto |
| qwen2.5-coder:1.5b | 986 MB | **Verificado** | **Más eficiente** |
| lfm2.5-thinking:latest | 731 MB | Disponible | Rápido |
| gemma3:1b | 815 MB | **Verificado** | **Ligero** |

*Nota: nomic-embed-text:v1.5 es modelo de embedding, no suitable para generación*

---

## Recomendaciones de Uso

### Para Código (Code Generation)
- **Recomendado**: `qwen2.5-coder:3b` o `qwen2.5-coder:1.5b`
- Especialización en generación de código

### Para Razonamiento
- **Recomendado**: `deepseek-r1:7b` o `deepseek-r1:1.5b`
- Chain-of-thought avanzado

### Para Eficiencia Máxima
- **Recomendado**: `qwen2.5-coder:1.5b` o `gemma3:1b`
- Menores recursos, mayor velocidad

### Para Capacidad Máxima
- **Recomendado**: `gemma4:e2b`
- Mayor contexto y complejidad

---

## Configuración de Contexto

Para mejor rendimiento con SML, configurar Ollama con contexto aumentado:

```bash
# En Ollama
/set parameter num_ctx 32768
/save model-name-32k
```

O vía variable de entorno:
```bash
OLLAMA_CONTEXT_LENGTH=32768 ollama serve
```