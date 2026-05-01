#!/usr/bin/env python3
"""
Fast Advanced SML Evaluation - Key Models
"""

import requests
import re
import json

OLLAMA_URL = "http://localhost:11434"

# Key models to test
MODELS = [
    ("qwen2.5-coder:1.5b", "986 MB", "Small Code"),
    ("gemma3:1b", "815 MB", "Lightweight"),
    ("qwen2.5-coder:3b", "1.9 GB", "Best Code"),
    ("qwen3:4b", "2.5 GB", "Fast Efficient"),
]

COMPLEX_PROMPT = """You are an autonomous SML agent.

Task:
1. Check if 'temp_data' directory exists
2. If not, create it with terminal command
3. Write math.py inside temp_data that calculates sqrt(144)

Write reasoning after '--- REASONING ---'
Write SML commands after '--- SML COMMANDS ---'

Format: @[tool:arg]"""

JSON_TOKENS = 45

def test_model(model_name, size, desc):
    print(f"\n>>> Testing {model_name}...")
    
    try:
        response = requests.post(
            f"{OLLAMA_URL}/api/generate",
            json={
                "model": model_name,
                "prompt": COMPLEX_PROMPT,
                "stream": False,
                "options": {"temperature": 0.2}
            },
            timeout=180
        )
        
        data = response.json()
        prompt_tokens = data.get("prompt_eval_count", 0)
        output_tokens = data.get("eval_count", 0)
        text = data.get("response", "")
        
        # Parse response
        if "--- SML COMMANDS ---" in text:
            reasoning, sml_part = text.split("--- SML COMMANDS ---", 1)
        else:
            reasoning, sml_part = text, ""
        
        # Extract SML commands
        commands = re.findall(r'@\[([a-z]+):([^\]]+)\]', text)
        sml_commands = [f"@[{t}:{a}]" for t, a in commands]
        
        # Calculate tokens
        action_tokens = sum(len(c.split()) for c in sml_commands)
        reasoning_tokens = output_tokens - action_tokens
        context_saved = JSON_TOKENS - action_tokens
        
        print(f"    Commands: {len(sml_commands)} | Action: {action_tokens} tokens | Saved: {context_saved}")
        
        return {
            "model": model_name,
            "commands": len(sml_commands),
            "action_tokens": action_tokens,
            "reasoning_tokens": reasoning_tokens,
            "context_saved": context_saved,
            "success": len(sml_commands) >= 2
        }
        
    except Exception as e:
        print(f"    ERROR: {str(e)[:50]}")
        return {"model": model_name, "error": str(e)}

print("="*60)
print("ADVANCED SML EVALUATION - Fast Version")
print("="*60)

results = []
for model, size, desc in MODELS:
    result = test_model(model, size, desc)
    results.append(result)

# Summary
print("\n" + "="*60)
print("SUMMARY")
print("="*60)
print(f"{'Model':<25} {'Cmds':>4} {'Action':>8} {'Saved':>8}")
print("-"*60)
for r in results:
    if "error" not in r:
        print(f"{r['model']:<25} {r['commands']:>4} {r['action_tokens']:>8} {r['context_saved']:>8}")
    else:
        print(f"{r['model']:<25} ERROR")

# Save
with open("fast_advanced_results.json", "w") as f:
    json.dump(results, f, indent=2)