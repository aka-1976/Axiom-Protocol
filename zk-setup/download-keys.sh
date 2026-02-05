#!/bin/bash
# AXIOM Protocol ZK Key Download Script
# Downloads proving keys from decentralized storage

set -e

echo "⬇️  AXIOM PROTOCOL ZK KEY DOWNLOADER"
echo "======================================"

# Configuration - Update these with actual hashes after ceremony
PROVING_KEY_IPFS="bafkreigjmiu2vtn7iehy6btmah7pfyxxknpddij4m3pyaq4occukv2qov4"  # Canonical IPFS CID
PROVING_KEY_SHA256="f6d4552e9be710535ef45a0470572311d7506a0f1f13b75f340bac11ccffd9cd"  # Trusted setup hash
VERIFICATION_KEY_URL="https://raw.githubusercontent.com/Ghost-84M/Axiom-Protocol/main/zk-setup/verification_key.json"

# Create keys directory
mkdir -p ~/.axiom/keys
cd ~/.axiom/keys

echo "📁 Keys will be stored in: $(pwd)"

# Check if IPFS is available
if command -v ipfs >/dev/null 2>&1; then
    IPFS_AVAILABLE=true
    echo "✅ IPFS client found"
else
    IPFS_AVAILABLE=false
    echo "⚠️  IPFS client not found - will use HTTP gateway"
fi

# Download verification key (small file from GitHub)
echo ""
echo "⬇️  Downloading verification key..."
if command -v curl >/dev/null 2>&1; then
    curl -L -o verification_key.json "$VERIFICATION_KEY_URL"
elif command -v wget >/dev/null 2>&1; then
    wget -O verification_key.json "$VERIFICATION_KEY_URL"
else
    echo "❌ Neither curl nor wget found. Please install one of them."
    exit 1
fi

# Verify verification key was downloaded
if [ ! -f "verification_key.json" ]; then
    echo "❌ Failed to download verification key"
    exit 1
fi

echo "✅ Verification key downloaded"

# Download proving key (large file from IPFS)
echo ""
echo "⬇️  Downloading proving key (this may take a while)..."

if [ "$IPFS_AVAILABLE" = true ]; then
    echo "📡 Using local IPFS client..."
    ipfs get "$PROVING_KEY_IPFS" -o proving_key.bin
else
    echo "🌐 Using IPFS HTTP gateway..."
    # Try multiple gateways for redundancy
    GATEWAYS=(
        "https://ipfs.io/ipfs/$PROVING_KEY_IPFS"
        "https://gateway.pinata.cloud/ipfs/$PROVING_KEY_IPFS"
        "https://cloudflare-ipfs.com/ipfs/$PROVING_KEY_IPFS"
    )

    DOWNLOADED=false
    for gateway in "${GATEWAYS[@]}"; do
        echo "Trying gateway: $gateway"
        if curl -L --max-time 300 -o proving_key.bin "$gateway" 2>/dev/null; then
            DOWNLOADED=true
            break
        fi
        echo "Gateway failed, trying next..."
    done

    if [ "$DOWNLOADED" = false ]; then
        echo "❌ Failed to download proving key from all gateways"
        echo "Please check your internet connection or try again later"
        exit 1
    fi
fi

# Verify proving key was downloaded and has correct hash
if [ ! -f "proving_key.bin" ]; then
    echo "❌ Proving key file not found after download"
    exit 1
fi

echo "🔍 Verifying proving key integrity..."
if command -v sha256sum >/dev/null 2>&1; then
    ACTUAL_HASH=$(sha256sum proving_key.bin | cut -d' ' -f1)
elif command -v shasum >/dev/null 2>&1; then
    ACTUAL_HASH=$(shasum -a 256 proving_key.bin | cut -d' ' -f1)
else
    echo "⚠️  No SHA256 tool found - skipping verification"
    ACTUAL_HASH="unknown"
fi

if [ "$ACTUAL_HASH" != "unknown" ] && [ "$ACTUAL_HASH" != "$PROVING_KEY_SHA256" ]; then
    echo "❌ Proving key hash mismatch!"
    echo "Expected: $PROVING_KEY_SHA256"
    echo "Actual:   $ACTUAL_HASH"
    echo "The downloaded file may be corrupted or tampered with."
    rm -f proving_key.bin
    exit 1
fi

# Get file sizes
PK_SIZE=$(ls -lh proving_key.bin | awk '{print $5}')
VK_SIZE=$(ls -lh verification_key.json | awk '{print $5}')

echo ""
echo "✅ DOWNLOAD COMPLETE!"
echo "===================="
echo ""
echo "📊 File sizes:"
echo "   - proving_key.bin: $PK_SIZE"
echo "   - verification_key.json: $VK_SIZE"
echo ""
echo "🔒 Verification:"
if [ "$ACTUAL_HASH" != "unknown" ]; then
    echo "   - Proving key SHA256: $ACTUAL_HASH ✓"
fi
echo ""
echo "📁 Keys stored in: $(pwd)"
echo ""
echo "🚀 Your node is now ready to generate and verify ZK-SNARK proofs!"
echo ""
echo "💡 Note: Keep these keys secure and backed up."