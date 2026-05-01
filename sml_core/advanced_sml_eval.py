#!/usr/bin/env python3
"""
Advanced SML Evaluation - Complex Multi-Step Agentic Tasks

Evaluates models on complex reasoning tasks vs SML command generation.
Metrics: Reasoning tokens, Action tokens, JSON penalty, Context saved.
"""

import requests
import re
import json

OLLAMA_URL = "http://localhost:11434"

# All available models (excluding embedding)
MODELS = [
    ("qwen2.5-coder:3b", "1.9 GB", "Code-specialized"),
    ("deepseek-r1:7b", "4.7 GB", "Reasoning"),
    ("gemma3:4b", "3.3 GB", "General"),
    ("gemma4:e2b", "7.2 GB", "Largest"),
    ("qwen3:4b", "2.5 GB", "Efficient"),
    ("cogito:3b", "2.2 GB", "Reasoning"),
    ("ministral-3:3b", "3.0 GB", "Performance"),
    ("llama3.2:3b", "2.0 GB", "Reliable"),
    ("nemotron-3-nano:4b", "2.8 GB", "Efficient"),
    ("granite3.1-moe:3b", "2.0 GB", "Code"),
    ("gemma3:1b", "815 MB", "Lightweight"),
    ("deepseek-r1:1.5b", "1.1 GB", "Compact"),
    ("qwen2.5-coder:1.5b", "986 MB", "Small"),
    ("lfm2.5-thinking:latest", "731 MB", "Fast"),
]

# Complex multi-step prompt
COMPLEX_PROMPT = """You are an autonomous SML agent connected to a zero-latency Rust dispatcher.

Task: Execute the following steps in order:
1. Check if a directory called temp_data exists in the current directory
2. If it doesn't exist, create it using the terminal
3. Write a python script inside temp_data called math.py that calculates the square root of 144 and prints it
4. Execute that python script

Write your thought process under the exact header '--- REASONING ---'. Then, write your SML pulses (commands) under the exact header '--- SML COMMANDS ---'.

Format your SML commands as: @[tool:argument]
Example: @[exist:temp_data] @[term:mkdir temp_data] @[write:temp_data/math.py|import math...]

IMPORTANT: Output ONLY the SML commands after '--- SML COMMANDS ---', no explanations."""

# JSON baseline for comparison (approximate tokens)
JSON_TEMPLATES = [
    '{"tool":"read","parameters":{"path":"temp_data"}}',
    '{"tool":"term","parameters":{"command":"mkdir temp_data"}}',
    '{"tool":"write","parameters":{"path":"temp_data/math.py","content":"import math\\nprint(int(math.sqrt(144)))"}}',
    '{"tool":"term","parameters":{"command":"python temp_data/math.py"}}',
]

JSON_TOKEN_COUNT = sum([len(t.split()) + 5 for t in JSON_TEMPLATES])  # ~45 tokens

def estimate_tokens(text: str) -> int:
    """Estimate token count (rough approximation: ~4 chars per token)"""
    return max(1, len(text) // 4)

def test_model(model_name: str, size: str, description: str) -> dict:
    """Test a single model with complex multi-step task"""
    print(f"\n{'='*70}")
    print(f"MODEL: {model_name} ({size}) - {description}")
    print(f"{'='*70}")
    print("Waiting for model to load...")
    
    result = {
        "model": model_name,
        "size": size,
        "description": description,
        "success": False,
        "reasoning_text": "",
        "sml_text": "",
        "sml_commands": [],
        "reasoning_tokens": 0,
        "action_tokens": 0,
        "json_penalty": JSON_TOKEN_COUNT,
        "context_saved": 0,
        "error": None,
        "prompt_tokens": 0,
        "output_tokens": 0,
    }
    
    try:
        # NO TIMEOUT - patient loading
        response = requests.post(
            f"{OLLAMA_URL}/api/generate",
            json={
                "model": model_name,
                "prompt": COMPLEX_PROMPT,
                "stream": False,
                "options": {
                    "temperature": 0.2,
                    "num_ctx": 4096,
                }
            }
        )
        
        data = response.json()
        
        # Get token metrics
        result["prompt_tokens"] = data.get("prompt_eval_count", 0)
        result["output_tokens"] = data.get("eval_count", 0)
        
        full_response = data.get("response", "")
        
        # SAFE PARSING: Split by plain text headers
        if "--- SML COMMANDS ---" in full_response:
            parts = full_response.split("--- SML COMMANDS ---")
            result["reasoning_text"] = parts[0] if len(parts) > 0 else ""
            result["sml_text"] = parts[1] if len(parts) > 1 else ""
        else:
            # Fallback: extract all @[...] commands
            result["reasoning_text"] = full_response
            result["sml_text"] = ""
        
        # Extract SML commands from any format
        pattern = r'@\[([a-z]+):([^\]]+)\]'
        matches = re.findall(pattern, result["sml_text"] + result["reasoning_text"])
        result["sml_commands"] = [f"@[{t}:{a}]" for t, a in matches]
        
        # Calculate reasoning tokens (everything except SML commands)
        clean_reasoning = re.sub(r'@\[.*?\]', '', result["reasoning_text"])
        result["reasoning_tokens"] = estimate_tokens(clean_reasoning)
        
        # Calculate action tokens (SML commands only)
        result["action_tokens"] = sum(estimate_tokens(cmd) for cmd in result["sml_commands"])
        
        # Calculate metrics
        result["context_saved"] = JSON_TOKEN_COUNT - result["action_tokens"]
        result["success"] = len(result["sml_commands"]) >= 3  # Need at least 3 commands
        
        # Print results
        print(f"\n--- EXTRACTION ---")
        print(f"Commands found: {len(result['sml_commands'])}")
        print(f"SML Commands: {result['sml_commands'][:5]}")
        
        print(f"\n--- TOKEN METRICS ---")
        print(f"Input Tokens (prompt): {result['prompt_tokens']}")
        print(f"Output Tokens (total): {result['output_tokens']}")
        print(f"Reasoning Tokens (analysis): {result['reasoning_tokens']}")
        print(f"Action Tokens (SML commands): {result['action_tokens']}")
        print(f"JSON Penalty (would be): {result['json_penalty']}")
        print(f"Context Saved: {result['context_saved']} tokens")
        
        if result["context_saved"] > 0:
            pct = (result["context_saved"] / result["json_penalty"]) * 100
            print(f"Efficiency Gain: {pct:.1f}%")
        
    except Exception as e:
        result["error"] = str(e)
        print(f"ERROR: {e}")
    
    return result

def main():
    print("="*70)
    print("ADVANCED SML EVALUATION - Complex Multi-Step Agentic Tasks")
    print("="*70)
    print(f"Ollama URL: {OLLAMA_URL}")
    print(f"\nTask: Check/create temp_data, write math.py, execute it")
    print(f"JSON Baseline: ~{JSON_TOKEN_COUNT} tokens")
    print(f"\nNOTE: Sequential testing - no timeouts")
    
    results = []
    
    for model, size, desc in MODELS:
        result = test_model(model, size, desc)
        results.append(result)
        
        # Small delay between models
        import time
        time.sleep(1)
    
    # Detailed Summary
    print("\n" + "="*70)
    print("DETAILED RESULTS")
    print("="*70)
    print(f"{'Model':<28} {'Cmds':>4} {'Reasoning':>10} {'Action':>8} {'Saved':>8} {'Status'}")
    print("-"*70)
    
    for r in results:
        status = "✓" if r["success"] else "✗"
        print(f"{r['model']:<28} {len(r['sml_commands']):>4} {r['reasoning_tokens']:>10} {r['action_tokens']:>8} {r['context_saved']:>8} {status}")
    
    # Best performers
    print("\n" + "="*70)
    print("BEST AGENTS (by context saved)")
    print("="*70)
    
    sorted_results = sorted(results, key=lambda x: x["context_saved"], reverse=True)
    for i, r in enumerate(sorted_results[:5], 1):
        print(f"{i}. {r['model']}: {r['context_saved']} tokens saved ({len(r['sml_commands'])} commands)")
    
    # Save results
    with open("advanced_sml_results.json", "w") as f:
        json.dump(results, f, indent=2, default=str)
    
    print(f"\n{'='*70}")
    print("COMPLETE")
    print(f"{'='*70}")
    print(f"Results saved to advanced_sml_results.json")

if __name__ == "__main__":
    main()