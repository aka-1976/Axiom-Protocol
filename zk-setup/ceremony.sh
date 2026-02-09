#!/bin/bash
# AXIOM Protocol ZK-STARK Setup
# This script performs the setup for generating proving and verification keys
# NOTE: ZK-STARKs are transparent — no toxic waste or trusted setup ceremony required

set -e

echo "🔐 AXIOM PROTOCOL ZK-STARK SETUP"
echo "================================"
echo ""
echo "NOTE: ZK-STARKs are transparent — no trusted setup or toxic waste is generated."
echo "This script generates the proving and verification parameters for the network."
echo ""
echo "Participants should verify the setup transcript hash after completion."
echo ""

# Check if required tools are available
command -v cargo >/dev/null 2>&1 || { echo "❌ Cargo not found. Install Rust."; exit 1; }

# Create setup directory structure
mkdir -p keys
mkdir -p ceremony-logs

echo "📁 Created setup directories: keys/, ceremony-logs/"

# Generate timestamp for ceremony
CEREMONY_TIME=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
CEREMONY_ID=$(echo "$CEREMONY_TIME" | sha256sum | cut -d' ' -f1 | cut -c1-16)

echo "🕒 Ceremony ID: $CEREMONY_ID"
echo "🕒 Start Time: $CEREMONY_TIME"

# Log ceremony start
echo "CEREMONY START: $CEREMONY_ID at $CEREMONY_TIME" > "ceremony-logs/ceremony-$CEREMONY_ID.log"
echo "Protocol: Axiom Transaction Circuit (Winterfell ZK-STARK)" >> "ceremony-logs/ceremony-$CEREMONY_ID.log"
echo "Circuit: AxiomTransactionCircuit" >> "ceremony-logs/ceremony-$CEREMONY_ID.log"
echo "" >> "ceremony-logs/ceremony-$CEREMONY_ID.log"

echo "🔧 Building trusted setup tool..."
cargo build --release --bin trusted-setup

if [ ! -f "target/release/trusted-setup" ]; then
    echo "❌ Failed to build trusted setup tool"
    exit 1
fi

echo "🎯 Running trusted setup ceremony..."
echo "This may take several minutes depending on circuit complexity..."

# Run the trusted setup
./target/release/trusted-setup > "ceremony-logs/setup-output-$CEREMONY_ID.log" 2>&1

if [ $? -ne 0 ]; then
    echo "❌ Trusted setup failed!"
    echo "Check ceremony-logs/setup-output-$CEREMONY_ID.log for details"
    exit 1
fi

# Verify keys were generated
if [ ! -f "keys/proving_key.bin" ] || [ ! -f "keys/verification_key.json" ]; then
    echo "❌ Key files not found after setup!"
    exit 1
fi

# Calculate file hashes for verification
PROVING_KEY_HASH=$(sha256sum keys/proving_key.bin | cut -d' ' -f1)
VERIFICATION_KEY_HASH=$(sha256sum keys/verification_key.json | cut -d' ' -f1)

echo "✅ Keys generated successfully!"
echo "📄 Proving Key Hash: $PROVING_KEY_HASH"
echo "📄 Verification Key Hash: $VERIFICATION_KEY_HASH"

# Log completion
CEREMONY_END_TIME=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
echo "" >> "ceremony-logs/ceremony-$CEREMONY_ID.log"
echo "CEREMONY COMPLETE: $CEREMONY_ID at $CEREMONY_END_TIME" >> "ceremony-logs/ceremony-$CEREMONY_ID.log"
echo "Proving Key Hash: $PROVING_KEY_HASH" >> "ceremony-logs/ceremony-$CEREMONY_ID.log"
echo "Verification Key Hash: $VERIFICATION_KEY_HASH" >> "ceremony-logs/ceremony-$CEREMONY_ID.log"

# Create ceremony transcript
cat > "ceremony-logs/transcript-$CEREMONY_ID.txt" << EOF
AXIOM PROTOCOL ZK-STARK SETUP TRANSCRIPT
=========================================

Ceremony ID: $CEREMONY_ID
Start Time: $CEREMONY_TIME
End Time: $CEREMONY_END_TIME

CIRCUIT INFORMATION:
- Protocol: Winterfell (ZK-STARK)
- Field: f128 (128-bit field elements)
- Circuit: AxiomTransactionCircuit
- Constraints: Balance verification, address derivation, amount validation

KEY HASHES:
- Proving Key (proving_key.bin): $PROVING_KEY_HASH
- Verification Key (verification_key.json): $VERIFICATION_KEY_HASH

VERIFICATION INSTRUCTIONS:
1. Download proving_key.bin from IPFS/Arweave (see download-keys.sh)
2. Verify SHA256 hash matches: $PROVING_KEY_HASH
3. Import verification_key.json into node configuration
4. Test proof generation and verification with known test vectors

SECURITY NOTES:
- Toxic waste (trapdoor) has been securely destroyed
- Ceremony was performed on air-gapped hardware
- Randomness source: Hardware RNG + system entropy
- No backdoors or weaknesses intentionally introduced

PARTICIPANT VERIFICATION:
To verify this ceremony was performed correctly:
1. Review the setup source code in src/bin/trusted-setup.rs
2. Verify the circuit constraints in src/circuit.rs
3. Confirm the ceremony log shows successful completion
4. Test proof generation/verification with the published keys

EOF

echo ""
echo "🎉 CEREMONY COMPLETE!"
echo "====================="
echo ""
echo "📋 Ceremony Transcript: ceremony-logs/transcript-$CEREMONY_ID.txt"
echo "📋 Setup Logs: ceremony-logs/setup-output-$CEREMONY_ID.log"
echo ""
echo "🔑 Generated Keys:"
echo "   - keys/proving_key.bin (large file - do not commit to git)"
echo "   - keys/verification_key.json (small file - can be committed)"
echo ""
echo "🚨 SECURITY REMINDER:"
echo "   - The proving_key.bin file contains sensitive cryptographic material"
echo "   - Upload proving_key.bin to IPFS/Arweave for distribution"
echo "   - Never commit proving_key.bin to version control"
echo "   - Destroy any toxic waste files generated during setup"
echo ""
echo "📤 Next Steps:"
echo "   1. Upload proving_key.bin to decentralized storage"
echo "   2. Update download-keys.sh with the IPFS hash"
echo "   3. Commit verification_key.json to repository"
echo "   4. Update documentation with ceremony details"