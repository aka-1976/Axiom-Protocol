#!/bin/bash

# Qubit Core - Decentralized 84M Launch Script
echo "--------------------------------------------------"
echo "🚀 INITIALIZING QUBIT CORE..."
echo "--------------------------------------------------"

# 1. Clean previous build artifacts
cargo clean

# 2. Build the optimized binary
echo "🛠️  Compiling release binary..."
cargo build --release

if [ $? -eq 0 ]; then
    echo "✅ Build Successful."
    echo "--------------------------------------------------"
    echo "🏛️  STARTING DECENTRALIZED NODE..."
    echo "--------------------------------------------------"
    
    # 3. Execute the binary
    # This runs the node and keeps it active in your terminal
    ./target/release/qubit
else
    echo "❌ Build Failed. Check the errors above."
    exit 1
fi
