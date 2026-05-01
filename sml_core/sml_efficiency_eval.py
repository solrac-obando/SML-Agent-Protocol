#!/usr/bin/env python3
"""
SML Efficiency Evaluation Script

Tests lightweight Ollama models with strict metrics:
- No timeouts (waits for model loading)
- Token efficiency metrics
- Pure SML syntax validation
"""

import requests
import re
import json

OLLAMA_URL = "http://localhost:11434"

MODELS = [
    ("gemma3:1b", "815 MB", "Lightweight Gemma"),
    ("qwen2.5-coder:1.5b", "986 MB", "Code-specialized small"),
    ("lfm2.5-thinking:latest", "731 MB", "Fast inference"),
]

PROMPT = "You are connected to SML protocol. Output ONLY: @[read:file.txt]"

def test_model(model_name: str, size: str, description: str) -> dict:
    """Test a single model - waits indefinitely for Ollama to load model"""
    print(f"\n{'='*60}")
    print(f"Testing: {model_name} ({size}) - {description}")
    print(f"{'='*60}")
    print(f"Waiting for Ollama to load model (this may take minutes)...")
    
    result = {
        "model": model_name,
        "size": size,
        "description": description,
        "success": False,
        "response": "",
        "sml_command": "",
        "is_pure_sml": False,
        "prompt_eval_count": None,
        "eval_count": None,
        "total_tokens": None,
        "error": None,
    }
    
    try:
        # NO TIMEOUT - waits as long as needed for model to load
        response = requests.post(
            f"{OLLAMA_URL}/api/generate",
            json={
                "model": model_name,
                "prompt": PROMPT,
                "stream": False,
                "options": {
                    "temperature": 0.1,
                    "num_ctx": 1024,
                }
            }
        )
        
        data = response.json()
        
        # Extract token metrics
        result["prompt_eval_count"] = data.get("prompt_eval_count", 0)
        result["eval_count"] = data.get("eval_count", 0)
        result["total_tokens"] = result["prompt_eval_count"] + result["eval_count"]
        
        # Get response text
        result["response"] = data.get("response", "")
        
        # Extract SML command
        pattern = r'@\[([a-z]+):([^\]]+)\]'
        matches = re.findall(pattern, result["response"])
        
        if matches:
            result["sml_command"] = f"@[{matches[0][0]}:{matches[0][1]}]"
            
            # Check if response is PURE SML (no extra characters)
            clean_response = result["response"].strip()
            sml = result["sml_command"]
            result["is_pure_sml"] = (
                clean_response == sml or
                (clean_response.startswith(sml) and len(clean_response) == len(sml) + 1)
            )
            result["success"] = True
        
        # Print results
        print(f"\nResponse: {result['response'][:100]}...")
        print(f"SML Command: {result['sml_command']}")
        print(f"Pure SML Syntax: {'YES ✓' if result['is_pure_sml'] else 'NO ✗'}")
        print(f"\n--- TOKEN METRICS ---")
        print(f"Input Tokens (prompt_eval_count): {result['prompt_eval_count']}")
        print(f"Output Tokens (eval_count): {result['eval_count']}")
        print(f"Total Tokens: {result['total_tokens']}")
        
    except Exception as e:
        result["error"] = str(e)
        print(f"ERROR: {e}")
    
    return result

def main():
    print("="*70)
    print("SML EFFICIENCY EVALUATION - Lightweight Models")
    print("="*70)
    print(f"Ollama URL: {OLLAMA_URL}")
    print(f"Prompt: {PROMPT}")
    print("\nNOTE: No timeouts - script will wait for model loading\n")
    
    results = []
    
    for model, size, desc in MODELS:
        result = test_model(model, size, desc)
        results.append(result)
    
    # Summary
    print("\n" + "="*70)
    print("FINAL SUMMARY")
    print("="*70)
    print(f"{'Model':<30} {'Output Tokens':<15} {'Pure SML':<10}")
    print("-"*70)
    
    for r in results:
        status = "✓ YES" if r["is_pure_sml"] else "✗ NO"
        tokens = str(r["eval_count"]) if r["eval_count"] else "N/A"
        print(f"{r['model']:<30} {tokens:<15} {status:<10}")
    
    print("-"*70)
    
    # Token efficiency comparison
    working = [r for r in results if r["success"] and r["is_pure_sml"]]
    if working:
        avg_tokens = sum(r["eval_count"] for r in working) / len(working)
        print(f"\nAverage Output Tokens (SML): {avg_tokens:.1f}")
        print(f"JSON Equivalent would be: ~15-20 tokens")
        print(f"Token Savings: {((20-avg_tokens)/20*100):.1f}%")
    
    # Save detailed results
    with open("sml_efficiency_results.json", "w") as f:
        json.dump(results, f, indent=2)
    print("\nResults saved to sml_efficiency_results.json")

if __name__ == "__main__":
    main()