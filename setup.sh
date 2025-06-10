#!/usr/bin/env bash
set -e

# Install Rust toolchain via rustup if cargo is missing
if ! command -v cargo >/dev/null 2>&1; then
    echo "Installing Rust toolchain via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

# Fetch project dependencies (can be run offline later)
cargo fetch

echo "Rust environment setup complete. You can now run 'cargo build' or 'cargo test'."