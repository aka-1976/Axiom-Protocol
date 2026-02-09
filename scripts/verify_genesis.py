#!/usr/bin/env python3
"""
verify_genesis.py — Axiom Protocol Genesis Pulse Verifier

Reads config/genesis_pulse.json, re-calculates its 512-bit BLAKE3-XOF
hash, and compares the result against the GENESIS_PULSE_HASH hardcoded
in src/lib.rs.

Usage:
    python3 scripts/verify_genesis.py

Requirements:
    pip install blake3
"""

import hashlib
import json
import os
import re
import sys

# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------

# Path to the genesis pulse file (relative to the repository root).
GENESIS_PULSE_PATH = os.path.join("config", "genesis_pulse.json")

# Path to the Rust source that contains GENESIS_PULSE_HASH.
LIB_RS_PATH = os.path.join("src", "lib.rs")

# If the script is invoked from a subdirectory, try to find the repo root.
def _find_repo_root() -> str:
    """Walk upward until we find Cargo.toml (repo root marker)."""
    path = os.path.abspath(os.path.dirname(__file__))
    for _ in range(10):
        if os.path.isfile(os.path.join(path, "Cargo.toml")):
            return path
        path = os.path.dirname(path)
    # Fallback: current working directory
    return os.getcwd()

REPO_ROOT = _find_repo_root()

# ---------------------------------------------------------------------------
# BLAKE3-512 (XOF) hashing
# ---------------------------------------------------------------------------

def blake3_512_hex(data: bytes) -> str:
    """Compute the 512-bit (64-byte) BLAKE3-XOF hash and return it as hex."""
    try:
        import blake3 as _blake3
        h = _blake3.blake3(data)
        return h.hexdigest(length=64)
    except ImportError:
        pass

    # Fallback: use the hashlib interface available in Python 3.11+ or
    # the blake3 PyPI package.  hashlib does not expose XOF for BLAKE3
    # so we require the dedicated package.
    print("ERROR: The 'blake3' Python package is required.")
    print("       Install it with:  pip install blake3")
    sys.exit(2)

# ---------------------------------------------------------------------------
# Extract expected hash from Rust source
# ---------------------------------------------------------------------------

def extract_expected_hash() -> str:
    """
    Parse src/lib.rs and extract the GENESIS_PULSE_HASH constant.

    The constant spans multiple lines using Rust's string continuation:
        pub const GENESIS_PULSE_HASH: &str =
            "3f178ac4...\\
             3a6b1524...";
    """
    lib_rs = os.path.join(REPO_ROOT, LIB_RS_PATH)
    if not os.path.isfile(lib_rs):
        print(f"ERROR: Cannot find {LIB_RS_PATH} (looked in {REPO_ROOT})")
        sys.exit(3)

    with open(lib_rs, "r") as f:
        source = f.read()

    # Match the constant across lines, capturing concatenated string pieces.
    pattern = r'pub\s+const\s+GENESIS_PULSE_HASH\s*:\s*&str\s*=\s*"([^"]+)"'
    # Because the string may span lines with backslash continuation, we
    # first collapse backslash-newline-whitespace sequences.
    collapsed = re.sub(r'\\\n\s*', '', source)
    match = re.search(pattern, collapsed)
    if not match:
        print("ERROR: Could not locate GENESIS_PULSE_HASH in src/lib.rs")
        sys.exit(3)

    return match.group(1).strip()

# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main() -> int:
    print("=" * 64)
    print("  AXIOM PROTOCOL — Genesis Pulse Verification")
    print("=" * 64)
    print()

    # 1. Locate and read genesis_pulse.json
    genesis_path = os.path.join(REPO_ROOT, GENESIS_PULSE_PATH)
    if not os.path.isfile(genesis_path):
        print(f"❌ INTEGRITY BREACH: {GENESIS_PULSE_PATH} not found!")
        return 1

    with open(genesis_path, "rb") as f:
        genesis_bytes = f.read()

    print(f"📄 File:     {GENESIS_PULSE_PATH}")
    print(f"   Size:     {len(genesis_bytes)} bytes")

    # 2. Compute BLAKE3-512 hash
    computed_hash = blake3_512_hex(genesis_bytes)
    print(f"🔑 Computed: {computed_hash}")

    # 3. Extract expected hash from source
    expected_hash = extract_expected_hash()
    print(f"📌 Expected: {expected_hash}")
    print()

    # 4. Compare
    if computed_hash == expected_hash:
        print("✅ FOUNDATION VERIFIED")
        print()
        print("   The genesis pulse file matches the hardcoded")
        print("   GENESIS_PULSE_HASH in src/lib.rs.")
        print("   The 124M supply chain is anchored to Block 0.")
        print()

        # Parse and display key genesis data for human verification.
        try:
            genesis = json.loads(genesis_bytes)
            print("   Genesis Summary:")
            print(f"     Protocol:  {genesis.get('protocol', 'N/A')}")
            print(f"     Version:   {genesis.get('version', 'N/A')}")
            supply = genesis.get("supply", {})
            print(f"     Supply:    {supply.get('total_supply_axm', 'N/A')} AXM")
            print(f"     Timestamp: {genesis.get('genesis_timestamp_utc', 'N/A')}")
        except json.JSONDecodeError:
            pass

        return 0
    else:
        print("❌ INTEGRITY BREACH")
        print()
        print("   The computed BLAKE3-512 hash does NOT match the")
        print("   expected GENESIS_PULSE_HASH in src/lib.rs.")
        print()
        print("   The genesis pulse file may have been tampered with.")
        print("   Do NOT trust this node's supply chain anchor.")
        return 1


if __name__ == "__main__":
    sys.exit(main())
