#!/bin/bash

echo "🔑 Generating ZK-SNARK Keys..."

# Create directories
mkdir -p zk-setup

# Run the trusted setup binary
cargo run --bin trusted-setup

echo "✅ ZK keys generated successfully!"
echo "📁 Keys saved to zk-setup/"
