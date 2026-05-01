#!/usr/bin/env python3
"""
Quick SML Protocol Verification - All Ollama Models
"""

import requests
import json
import re
import time

OLLAMA_URL = "http://localhost:11434"

MODELS = [
    ("qwen2.5-coder:3b", "1.9 GB", "Code"),
    ("deepseek-r1:7b", "4.7 GB", "Reasoning"),
    ("gemma3:4b", "3.3 GB", "General"),
    ("gemma4:e2b", "7.2 GB", "Large"),
    ("qwen3:4b", "2.5 GB", "Efficient"),
    ("cogito:3b", "2.2 GB", "Reasoning"),
    ("ministral-3:3b", "3.0 GB", "Performance"),
    ("llama3.2:3b", "2.0 GB", "Reliable"),
    ("nemotron-3-nano:4b", "2.8 GB", "Efficient"),
    ("granite3.1-moe:3b", "2.0 GB", "Code"),
    ("gemma3:1b", "815 MB", "Light"),
    ("deepseek-r1:1.5b", "1.1 GB", "Compact"),
    ("qwen2.5-coder:1.5b", "986 MB", "Small"),
    ("lfm2.5-thinking:latest", "731 MB", "Fast"),
]

def quick_test(model: str, size: str, category: str):
    prompt = "Output ONLY: @[read:file.txt]"
    
    try:
        start = time.time()
        resp = requests.post(f"{OLLAMA_URL}/api/generate", json={
            "model": model,
            "prompt": prompt,
            "stream": False,
            "options": {"temperature": 0.1, "num_ctx": 1024}
        }, timeout=30)
        elapsed = time.time() - start
        
        text = resp.json().get("response", "")
        
        # Check if SML command is generated
        pattern = r'@\[([a-z]+):([^\]]+)\]'
        matches = re.findall(pattern, text)
        
        if matches:
            return "✓ WORKS", elapsed
        else:
            return "~ NO_SML", elapsed
            
    except Exception as e:
        return f"✗ {str(e)[:30]}", 0

print("=" * 70)
print("SML Protocol Quick Verification - All Ollama Models")
print("=" * 70)
print()

results = []
for model, size, category in MODELS:
    status, elapsed = quick_test(model, size, category)
    print(f"{status:15} | {model:30} | {size:8} | {elapsed:.1f}s" if elapsed else f"{status:15} | {model:30} | {size:8}")
    results.append((model, status))

print()
working = sum(1 for _, s in results if "WORKS" in s)
print(f"Working: {working}/{len(results)}")
print()
print("Full results saved to test_results.json")

with open("test_results.json", "w") as f:
    json.dump(results, f, indent=2)