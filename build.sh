#!/bin/bash

# ProjectBoard CLI Build Script

echo "🚀 Building ProjectBoard CLI..."

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust/Cargo not found. Please install Rust first:"
    echo "   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Build the project
echo "🔨 Compiling..."
if cargo build --release; then
    echo "✅ Build successful!"
    echo "📦 Binary location: target/release/pb"
    echo ""
    echo "To install globally:"
    echo "  cp target/release/pb /usr/local/bin/"
    echo "  # or"
    echo "  cargo install --path ."
    echo ""
    echo "To get started:"
    echo "  cd your-git-repository"
    echo "  pb init"
    echo "  pb add \"Your first task\""
else
    echo "❌ Build failed"
    exit 1
fi
