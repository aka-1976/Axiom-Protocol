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
