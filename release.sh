#!/bin/bash

# Exit on any error
set -e

echo "🚀 Building easy-ssh in release mode..."
cargo build --release

echo "📦 Installing easy-ssh globally to ~/.cargo/bin..."
# Make sure previous versions are overwritten
cargo install --path . --force

echo ""
echo "✅ Successfully built and installed easy-ssh!"
echo "You can now run 'easy-ssh' from any terminal."
