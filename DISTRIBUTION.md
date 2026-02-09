# DISTRIBUTION.md — Weight Verification for Axiom Protocol

## Overview

The `weights.bin` file contains the production NeuralGuardian model
weights used by every Axiom node for AI-powered threat detection. It is
a **signed release artifact** — its SHA-256 fingerprint is hardcoded into
the protocol as `GENESIS_WEIGHTS_HASH`.

When the node starts, it computes the SHA-256 of the local `weights.bin`
and compares it against `GENESIS_WEIGHTS_HASH`. If the hashes do not
match, the node **refuses to start** (fatal exit). This ensures that
every node on the 124M supply network is running identical, auditable AI
logic.

## Verifying weights.bin Before First Run

Before launching your node for the first time, verify that your local
copy of `weights.bin` matches the genesis anchor:

```bash
# 1. Compute the SHA-256 hash of your local weights.bin
sha256sum weights.bin

# 2. Compare the output against the genesis anchor printed by the node:
#    e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
#
#    (This value is defined as GENESIS_WEIGHTS_HASH in src/lib.rs)
```

If the hashes match, your `weights.bin` is authentic. If they differ,
**do not run the node** — obtain a fresh copy from the official release.

## Where to Obtain weights.bin

Official release artifacts (including `weights.bin`) are published
alongside each tagged release in the
[Axiom-Protocol GitHub Releases](https://github.com/Ghost-84M/Axiom-Protocol/releases)
page.

Each release includes:
- The `weights.bin` binary
- The expected SHA-256 hash in the release notes
- A GPG signature for additional verification

## Why This Matters

The NeuralGuardian AI is responsible for peer trust scoring and threat
detection across the decentralised network. If an attacker could modify
the model weights, they could:

- Cause legitimate peers to be banned (denial of service)
- Allow malicious peers to bypass threat detection
- Compromise the 124M supply integrity

The `GENESIS_WEIGHTS_HASH` anchoring prevents all of these attacks by
making model tampering detectable before the node even joins the network.

## Genesis Pulse Fingerprint

The `config/genesis_pulse.json` file anchors the entire tamper-evident
pulse chain to the absolute start of the 124M network. Its 512-bit
BLAKE3 (XOF) fingerprint is hardcoded as `GENESIS_PULSE_HASH` in
`src/lib.rs`.

```bash
# Verify the genesis pulse (requires b3sum or python3 with blake3):
b3sum --raw --length 64 config/genesis_pulse.json | xxd -p -c 128

# Or with Python:
python3 -c "import blake3; print(blake3.blake3(open('config/genesis_pulse.json','rb').read()).hexdigest(length=64))"

# Expected (512-bit / 128 hex chars):
# 3f178ac4d3e0155210addeb1433f588ef12ce5a6a811ed8c77fca5ffd33726943a6b152420c7b2d611fb187cfd26390e18ad4df0947fea0060dab8b75007de74
```

If the hash does not match, the node will print `GENESIS PULSE INTEGRITY
FAILURE` and exit. See also `docs/GENESIS_RECORD.md` for the full raw
JSON content of this file.
