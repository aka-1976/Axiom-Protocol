# MANIFEST.md — Axiom Protocol Verification Manifest

This document provides the exact commands needed to verify the integrity of
every critical artifact before running an Axiom node. The goal is to transition
from **"trust us"** to **"verify the manifest."**

---

## 1. Verify `weights.bin` (NeuralGuardian AI Model)

The `weights.bin` file contains the NeuralGuardian model weights. Every
Axiom node verifies this file at startup against the `GENESIS_WEIGHTS_HASH`
constant defined in `src/lib.rs`. If the hash doesn't match, the node
refuses to start.

### Current Genesis Weights Hash

```text
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
```

### Verification Commands

**Using `sha256sum` (Linux / macOS with coreutils):**

```bash
sha256sum weights.bin
# Expected output:
# e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  weights.bin
```

**Using `openssl` (cross-platform):**

```bash
openssl dgst -sha256 weights.bin
# Expected output:
# SHA2-256(weights.bin)= e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
```

**Using `shasum` (macOS built-in):**

```bash
shasum -a 256 weights.bin
```

If the output does **not** match the hash above, **do not run the node**.
Your `weights.bin` has been tampered with or is from a different release.

---

## 2. Verify the `axiom-node` Binary

After building with `cargo build --release`, verify the binary has not been
modified in transit.

### Compute the SHA-256 of your local binary

```bash
sha256sum target/release/axiom-node
```

### Compare against the official release hash

Each GitHub Release includes a `SHA256SUMS` file. Download it and compare:

```bash
# Download the checksums file from the release
curl -sL https://github.com/Ghost-84M/Axiom-Protocol/releases/latest/download/SHA256SUMS -o SHA256SUMS

# Verify your binary
sha256sum -c SHA256SUMS --ignore-missing
# Expected output:
# target/release/axiom-node: OK
```

**Using `openssl`:**

```bash
openssl dgst -sha256 target/release/axiom-node
```

---

## 3. Verify `config/genesis_pulse.json` (Genesis Pulse Anchor)

The genesis pulse is the cryptographic origin of the tamper-evident pulse
chain. The node verifies it at startup against `GENESIS_PULSE_HASH` in
`src/lib.rs`.

### Verification Command

```bash
# Compute 512-bit BLAKE3 hash (requires b3sum: cargo install b3sum)
b3sum --num-bytes 64 config/genesis_pulse.json

# Or using Python with the blake3 package (pip install blake3):
python3 -c "
import blake3
data = open('config/genesis_pulse.json', 'rb').read()
h = blake3.blake3(data)
print(h.hexdigest(length=64))
"
```

Compare the output against the `GENESIS_PULSE_HASH` constant in `src/lib.rs`.

---

## 4. Verify the Genesis Anchor

The 512-bit BLAKE3 Genesis Anchor is hardcoded in `src/genesis.rs` as
`GENESIS_ANCHOR_512`. It is computed from the string:

```text
Axiom V4.0.0: Fully Decentralized. Non-Governance. Built for the World.
```

### Independent Verification

```bash
echo -n "Axiom V4.0.0: Fully Decentralized. Non-Governance. Built for the World." | b3sum --num-bytes 64
```

---

## 5. Build from Source (Reproducible Build)

For maximum trust, build the binary yourself:

```bash
git clone https://github.com/Ghost-84M/Axiom-Protocol.git
cd Axiom-Protocol
cargo build --release
sha256sum target/release/axiom-node
```

Compare the resulting hash against the official release's `SHA256SUMS`.

---

## Summary of Critical Constants

| Artifact               | Hash Algorithm | Constant in Source        | File              |
|------------------------|----------------|---------------------------|-------------------|
| NeuralGuardian weights | SHA-256        | `GENESIS_WEIGHTS_HASH`    | `src/lib.rs`      |
| Genesis Pulse          | BLAKE3-512     | `GENESIS_PULSE_HASH`      | `src/lib.rs`      |
| Genesis Anchor         | BLAKE3-512     | `GENESIS_ANCHOR_512`      | `src/genesis.rs`  |

---

*Last updated: 2026-02-09 — Axiom Protocol v4.1.0*
