#!/usr/bin/env python3
"""
Live test of SML Protocol with Ollama

This script demonstrates the SML (Symbolic Micro-Language) protocol
by sending a request to Ollama and executing the returned SML commands.
"""

import requests
import json
import subprocess
import sys
import re

OLLAMA_URL = "http://localhost:11434"
MODEL = "qwen2.5-coder:3b"  # Good for code tasks

def chat_with_ollama(prompt: str) -> str:
    """Send a chat request to Ollama"""
    response = requests.post(
        f"{OLLAMA_URL}/api/generate",
        json={
            "model": MODEL,
            "prompt": prompt,
            "stream": False,
            "options": {
                "num_ctx": 4096,
                "temperature": 0.1,
            }
        },
        timeout=60
    )
    return response.json().get("response", "")

def parse_sml_command(text: str) -> list:
    """Parse SML commands from text using regex"""
    pattern = r'@\[([a-z]+):([^\]]+)\]'
    matches = re.findall(pattern, text)
    return [(tool, args) for tool, args in matches]

def execute_sml_command(tool: str, args: str) -> str:
    """Execute an SML command using the sml_core binary"""
    cmd = f"@[{'{}'}:{'{}'}]".format(tool, args)
    result = subprocess.run(
        ["./target/release/sml_core", "--execute", cmd],
        capture_output=True,
        text=True,
        cwd="/home/carlosobando/proyectos_IA/microlenguaje-IA-instintivo/sml_core"
    )
    return result.stdout.strip()

def main():
    print("=" * 60)
    print("SML Protocol Live Test with Ollama")
    print("=" * 60)
    print()

    # Test 1: Simple read command
    print("Test 1: Ask Ollama to read a file")
    print("-" * 40)
    prompt1 = """You are connected to the SML protocol.
Use ONLY the format @[read:path] to read a file.

Read the Cargo.toml file and tell me the project name.
Respond with ONLY the SML command, nothing else."""

    response1 = chat_with_ollama(prompt1)
    print(f"Ollama response: {response1[:200]}...")
    
    commands = parse_sml_command(response1)
    print(f"Parsed commands: {commands}")
    
    if commands:
        tool, args = commands[0]
        result = execute_sml_command(tool, args)
        print(f"Execution result: {result[:200]}...")
    print()

    # Test 2: Write command
    print("Test 2: Ask Ollama to write a file")
    print("-" * 40)
    prompt2 = """You are connected to the SML protocol.
Use ONLY the format @[write:path|content] to write a file.

Write a simple hello world Python file called "hello.py" with: print("Hello from SML!")
Respond with ONLY the SML command, nothing else."""

    response2 = chat_with_ollama(prompt2)
    print(f"Ollama response: {response2[:200]}...")
    
    commands2 = parse_sml_command(response2)
    print(f"Parsed commands: {commands2}")
    
    if commands2:
        tool, args = commands2[0]
        parts = args.split("|", 1)
        if len(parts) == 2:
            result = execute_sml_command(tool, args)
            print(f"Execution result: {result}")
    print()

    # Test 3: Terminal command
    print("Test 3: Ask Ollama to run a terminal command")
    print("-" * 40)
    prompt3 = """You are connected to the SML protocol.
Use ONLY the format @[term:command] to run a terminal command.

Run "echo SML Protocol Works!" in the terminal.
Respond with ONLY the SML command, nothing else."""

    response3 = chat_with_ollama(prompt3)
    print(f"Ollama response: {response3[:200]}...")
    
    commands3 = parse_sml_command(response3)
    print(f"Parsed commands: {commands3}")
    
    if commands3:
        tool, args = commands3[0]
        result = execute_sml_command(tool, args)
        print(f"Execution result: {result}")
    print()

    print("=" * 60)
    print("Live test completed!")
    print("=" * 60)

if __name__ == "__main__":
    main()