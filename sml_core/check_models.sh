#!/bin/bash
# Quick model verification - one at a time with short timeout

echo "============================================================"
echo "SML Protocol - Quick Model Verification"
echo "============================================================"
echo ""

MODELS=(
    "qwen2.5-coder:3b"
    "deepseek-r1:7b"
    "gemma3:4b"
    "gemma4:e2b"
    "qwen3:4b"
    "cogito:3b"
    "ministral-3:3b"
    "llama3.2:3b"
    "nemotron-3-nano:4b"
    "granite3.1-moe:3b"
    "gemma3:1b"
    "deepseek-r1:1.5b"
    "qwen2.5-coder:1.5b"
    "lfm2.5-thinking:latest"
)

PROMPT='Output ONLY: @[read:file.txt]'

for MODEL in "${MODELS[@]}"; do
    echo -n "Testing $MODEL... "
    
    RESULT=$(curl -s --max-time 15 http://localhost:11434/api/generate \
        -d "{\"model\":\"$MODEL\",\"prompt\":\"$PROMPT\",\"stream\":false}" 2>/dev/null)
    
    if [ $? -eq 0 ] && [ -n "$RESULT" ]; then
        if echo "$RESULT" | grep -q '@\[read:'; then
            echo "✓ WORKS"
        else
            echo "~ Generates text (no SML detected)"
        fi
    else
        echo "✗ Timeout/Error"
    fi
done

echo ""
echo "Done!"