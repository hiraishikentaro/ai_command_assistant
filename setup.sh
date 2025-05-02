#!/bin/bash

# Setup script for AI Command Assistant (ACliA)
# This script helps to configure the OpenAI API key and other settings

echo "AI Command Assistant (ACliA) Setup"
echo "=================================="
echo

# Check if the config directory exists, if not create it
CONFIG_DIR="$HOME/.config/acia"
if [ ! -d "$CONFIG_DIR" ]; then
    echo "Creating config directory at $CONFIG_DIR"
    mkdir -p "$CONFIG_DIR"
fi

# Setup config file
CONFIG_FILE="$CONFIG_DIR/config.toml"

# Ask for OpenAI API key
echo "Please enter your OpenAI API key (starts with 'sk-'):"
read -s OPENAI_API_KEY
echo

if [ -z "$OPENAI_API_KEY" ]; then
    echo "Error: API key is required. Exiting."
    exit 1
fi

# Ask for preferred model
echo "Select your preferred model (default: gpt-3.5-turbo):"
echo "1. gpt-3.5-turbo (faster, cheaper)"
echo "2. gpt-4 (more capable, more expensive)"
read -p "Enter your choice [1]: " MODEL_CHOICE
echo

MODEL="gpt-3.5-turbo"
if [ "$MODEL_CHOICE" = "2" ]; then
    MODEL="gpt-4"
fi

# Ask for temperature
echo "Set the temperature (0.0-1.0, lower is more precise, higher is more creative)"
read -p "Enter temperature [0.2]: " TEMPERATURE
echo

if [ -z "$TEMPERATURE" ]; then
    TEMPERATURE="0.2"
fi

# Create or update config file
cat > "$CONFIG_FILE" << EOF
# AI Command Assistant Configuration

# OpenAI API Key
openai_api_key = "$OPENAI_API_KEY"

# LLM Settings
model = "$MODEL"
temperature = $TEMPERATURE
max_tokens = 500
timeout_seconds = 30
EOF

echo "Configuration saved to $CONFIG_FILE"
echo
echo "You can also set the API key as an environment variable:"
echo "export ACIA_OPENAI_API_KEY='$OPENAI_API_KEY'"
echo
echo "Setup complete! You can now run ACliA."
