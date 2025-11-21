#!/bin/bash
set -e

echo "╔═══════════════════════════════════════════════════════════╗"
echo "║  Installing Scalpel v1.0.0 - Kernel Surgeon Edition      ║"
echo "╚═══════════════════════════════════════════════════════════╝"
echo ""

# Check for Rust
if ! command -v cargo &> /dev/null; then
    echo "Error: Rust is not installed."
    echo "Please install Rust from https://rustup.rs/"
    exit 1
fi

echo "Building Scalpel (release mode)..."
cargo build --release

echo ""
echo "Installation options:"
echo "  1. Install to /usr/local/bin (requires sudo)"
echo "  2. Install to ~/.local/bin (user install)"
echo "  3. Skip installation"
echo ""
read -p "Choose option (1-3): " choice

case $choice in
    1)
        echo "Installing to /usr/local/bin..."
        sudo cp target/release/scalpel /usr/local/bin/
        sudo chmod +x /usr/local/bin/scalpel
        echo "✓ Scalpel installed to /usr/local/bin/scalpel"
        ;;
    2)
        echo "Installing to ~/.local/bin..."
        mkdir -p ~/.local/bin
        cp target/release/scalpel ~/.local/bin/
        chmod +x ~/.local/bin/scalpel
        echo "✓ Scalpel installed to ~/.local/bin/scalpel"
        echo ""
        echo "Note: Make sure ~/.local/bin is in your PATH"
        echo "Add this to your ~/.bashrc or ~/.zshrc:"
        echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
        ;;
    3)
        echo "Skipping installation. Binary is at: target/release/scalpel"
        ;;
    *)
        echo "Invalid option. Skipping installation."
        ;;
esac

echo ""
echo "Setup configuration:"
read -p "Do you have an Anthropic API key? (y/n): " has_key

if [ "$has_key" = "y" ]; then
    read -p "Enter your Anthropic API key: " api_key

    mkdir -p ~/.config/scalpel
    cat > ~/.config/scalpel/config.toml <<EOF
# Scalpel Configuration
api_key = "$api_key"
model = "claude-3-5-sonnet-20241022"
endpoint = "https://api.anthropic.com/v1"
safety_guards = true
require_confirmation = true
max_probes = 16
log_level = "info"
EOF

    echo "✓ Configuration saved to ~/.config/scalpel/config.toml"
else
    echo ""
    echo "You can set your API key later with:"
    echo "  export ANTHROPIC_API_KEY='your-key-here'"
    echo "Or create a config file at ~/.config/scalpel/config.toml"
fi

echo ""
echo "╔═══════════════════════════════════════════════════════════╗"
echo "║  Installation Complete!                                   ║"
echo "╚═══════════════════════════════════════════════════════════╝"
echo ""
echo "Quick start:"
echo "  sudo scalpel                    # Launch interactive mode"
echo "  sudo scalpel --help             # Show help"
echo ""
echo "For more information, see the README.md"
echo ""
echo "⚠️  Remember: Scalpel requires root or CAP_BPF capability"
