#!/bin/bash

# Exit on any error
set -e

echo "🚀 Building easy-ssh-tui in release mode..."
cargo build --release

echo "📦 Installing easy-ssh-tui globally to ~/.cargo/bin..."
# Make sure previous versions are overwritten
cargo install --path . --force

echo ""
echo "✅ Successfully built and installed easy-ssh-tui!"
echo "You can now run 'easy-ssh-tui' from any terminal."
