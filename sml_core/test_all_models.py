#!/usr/bin/env python3
"""
Comprehensive SML Protocol Verification with All Local Ollama Models

This script tests the SML protocol with all locally installed Ollama models.
"""

import requests
import json
import subprocess
import sys
import re
import time
from concurrent.futures import ThreadPoolExecutor, as_completed

OLLAMA_URL = "http://localhost:11434"

# All available models (excluding embedding models)
MODELS = [
    ("qwen2.5-coder:3b", "1.9 GB", "Best for code tasks"),
    ("deepseek-r1:7b", "4.7 GB", "Excellent reasoning"),
    ("gemma3:4b", "3.3 GB", "Good general purpose"),
    ("gemma4:e2b", "7.2 GB", "Largest capacity"),
    ("qwen3:4b", "2.5 GB", "Fast, efficient"),
    ("cogito:3b", "2.2 GB", "Good reasoning"),
    ("ministral-3:3b", "3.0 GB", "Good performance"),
    ("llama3.2:3b", "2.0 GB", "Reliable"),
    ("nemotron-3-nano:4b", "2.8 GB", "Efficient"),
    ("granite3.1-moe:3b", "2.0 GB", "Good for code"),
    ("gemma3:1b", "815 MB", "Lightweight"),
    ("deepseek-r1:1.5b", "1.1 GB", "Compact"),
    ("qwen2.5-coder:1.5b", "986 MB", "Small but capable"),
    ("lfm2.5-thinking:latest", "731 MB", "Fast inference"),
]

# Models that are embedding models only (should fail for generation)
SKIP_MODELS = ["nomic-embed-text:v1.5"]

def test_model(model_name: str, size: str, notes: str) -> dict:
    """Test a single model with SML protocol"""
    result = {
        "model": model_name,
        "size": size,
        "notes": notes,
        "status": "PENDING",
        "response": "",
        "sml_commands": [],
        "execution_results": [],
        "error": None,
    }

    try:
        # Simple prompt asking for SML command
        prompt = """You are connected to the SML protocol.
Use ONLY the format @[read:filename] to read a file.
Read the file called "test.txt".
Respond with ONLY the SML command, nothing else."""

        start_time = time.time()
        
        response = requests.post(
            f"{OLLAMA_URL}/api/generate",
            json={
                "model": model_name,
                "prompt": prompt,
                "stream": False,
                "options": {
                    "num_ctx": 4096,
                    "temperature": 0.1,
                    "timeout": 120,
                }
            },
            timeout=120
        )
        
        elapsed = time.time() - start_time
        
        if response.status_code != 200:
            result["error"] = f"HTTP {response.status_code}"
            result["status"] = "ERROR"
            return result

        response_text = response.json().get("response", "")
        result["response"] = response_text[:500]
        result["elapsed"] = f"{elapsed:.2f}s"

        # Parse SML commands
        pattern = r'@\[([a-z]+):([^\]]+)\]'
        matches = re.findall(pattern, response_text)
        result["sml_commands"] = [(tool, args) for tool, args in matches]

        if matches:
            result["status"] = "SML_GENERATED"
            
            # Execute the first command
            tool, args = matches[0]
            exec_result = execute_sml_command(tool, args)
            result["execution_results"].append({
                "command": f"@[{tool}:{args}]",
                "result": exec_result[:200]
            })
            
            if "test.txt" in exec_result or "[ERR" in exec_result:
                result["status"] = "WORKS"
            else:
                result["status"] = "PARTIAL"
        else:
            result["status"] = "NO_SML"
            result["error"] = "No SML command detected in response"

    except requests.exceptions.Timeout:
        result["status"] = "TIMEOUT"
        result["error"] = "Request timed out after 120s"
    except Exception as e:
        result["status"] = "ERROR"
        result["error"] = str(e)

    return result

def execute_sml_command(tool: str, args: str) -> str:
    """Execute an SML command using the sml_core binary"""
    cmd = f"@[{tool}:{args}]"
    try:
        result = subprocess.run(
            ["./target/release/sml_core", "--execute", cmd],
            capture_output=True,
            text=True,
            timeout=10,
            cwd="/home/carlosobando/proyectos_IA/microlenguaje-IA-instintivo/sml_core"
        )
        return result.stdout.strip()
    except Exception as e:
        return f"[ERROR] {str(e)}"

def print_result(result: dict):
    """Print test result in a formatted way"""
    status_icon = {
        "WORKS": "✓",
        "SML_GENERATED": "~",
        "PARTIAL": "⚠",
        "NO_SML": "✗",
        "ERROR": "✗",
        "TIMEOUT": "⏱",
        "PENDING": "...",
    }.get(result["status"], "?")

    print(f"  {status_icon} {result['model']}")
    print(f"      Size: {result.get('size', 'N/A')}")
    print(f"      Status: {result['status']}")
    
    if result.get("elapsed"):
        print(f"      Time: {result['elapsed']}")
    
    if result.get("sml_commands"):
        print(f"      Commands: {result['sml_commands'][:3]}")
    
    if result.get("error"):
        print(f"      Error: {result['error']}")

def main():
    print("=" * 70)
    print("SML Protocol Comprehensive Verification - All Ollama Models")
    print("=" * 70)
    print()
    print(f"Total models to test: {len(MODELS)}")
    print(f"Ollama URL: {OLLAMA_URL}")
    print()

    # Check if Ollama is available
    try:
        response = requests.get(f"{OLLAMA_URL}/api/tags", timeout=5)
        available_models = [m["name"] for m in response.json().get("models", [])]
        print(f"Available Ollama models: {len(available_models)}")
    except Exception as e:
        print(f"ERROR: Cannot connect to Ollama: {e}")
        return 1

    print()
    print("Running tests...")
    print("-" * 70)

    results = []
    for model, size, notes in MODELS:
        print(f"\nTesting: {model} ({size})")
        result = test_model(model, size, notes)
        results.append(result)
        print_result(result)

    # Summary
    print()
    print("=" * 70)
    print("SUMMARY")
    print("=" * 70)

    status_counts = {}
    for r in results:
        status_counts[r["status"]] = status_counts.get(r["status"], 0) + 1

    for status, count in sorted(status_counts.items()):
        icon = {
            "WORKS": "✓",
            "SML_GENERATED": "~",
            "PARTIAL": "⚠",
            "NO_SML": "✗",
            "ERROR": "✗",
            "TIMEOUT": "⏱",
        }.get(status, "?")
        print(f"  {icon} {status}: {count}")

    print()
    working = status_counts.get("WORKS", 0)
    total = len(results)
    print(f"Total working: {working}/{total} ({100*working/total:.0f}%)")
    print()

    # Save results to file
    with open("test_results.json", "w") as f:
        json.dump(results, f, indent=2)
    print("Results saved to test_results.json")

    return 0 if working > 0 else 1

if __name__ == "__main__":
    sys.exit(main())