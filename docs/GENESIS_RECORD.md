# GENESIS_RECORD.md — Axiom Protocol Block 0 Parameters

This document contains the full raw JSON of `config/genesis_pulse.json`
so that auditors can verify the "Second Zero" parameters of the 124M
supply without cloning the repository.

## BLAKE3-512 Fingerprint

```
3f178ac4d3e0155210addeb1433f588ef12ce5a6a811ed8c77fca5ffd33726943a6b152420c7b2d611fb187cfd26390e18ad4df0947fea0060dab8b75007de74
```

This value is hardcoded as `GENESIS_PULSE_HASH` in `src/lib.rs`.

## Genesis Pulse JSON

```json
{
    "protocol": "Axiom Protocol",
    "version": "4.1.0",
    "genesis_block": {
        "parent": "0000000000000000000000000000000000000000000000000000000000000000",
        "slot": 0,
        "miner": "0000000000000000000000000000000000000000000000000000000000000000",
        "transactions": [],
        "vdf_proof": "0000000000000000000000000000000000000000000000000000000000000000",
        "nonce": 0
    },
    "genesis_anchor_512": "87da3627016686eda1df67317238cfd88dbb631f541811d84e9018bfb508cddb2a8fa192bdf16c4bb5f191154d0165cd6b6acb22918353b786b5c100be7e89dc",
    "genesis_timestamp": 1737331200,
    "genesis_timestamp_utc": "2025-01-20T00:00:00Z",
    "supply": {
        "total_supply_units": 12400000000000000,
        "total_supply_axm": "124,000,000.000000",
        "initial_reward_units": 5000000000,
        "initial_reward_axm": "5,000.000000",
        "halving_interval_blocks": 1240000,
        "block_time_seconds": 1800,
        "smallest_unit": 1000000
    },
    "network": {
        "protocol_name": "Axiom",
        "ticker": "AXM",
        "consensus": "VDF + PoW Hybrid",
        "vdf_time_lock_seconds": 1800,
        "hash_algorithm": "BLAKE3-512 (XOF mode)",
        "zk_proof_system": "ZK-STARK (512-bit)",
        "ai_guardian": "NeuralGuardian v1.0"
    },
    "declaration": "Axiom V4.0.0: Fully Decentralized. Non-Governance. Built for the World."
}
```

## Key Parameters

| Parameter | Value | Description |
|---|---|---|
| Total Supply | 124,000,000 AXM | Immutable 124M cap enforced by ZK-STARK proofs |
| Smallest Unit | 10⁻⁶ AXM | 1,000,000 units = 1 AXM |
| Initial Reward | 5,000 AXM | First-era mining reward per block |
| Halving Interval | 1,240,000 blocks | Reward halves every 1.24M blocks |
| Block Time | 1,800 seconds | 30-minute VDF time-lock |
| Genesis Timestamp | 2025-01-20T00:00:00Z | Unix 1737331200 |
| Hash Algorithm | BLAKE3-512 (XOF) | 64-byte extended output for post-quantum margin |
| Consensus | VDF + PoW Hybrid | Sequential proof-of-time prevents GPU attacks |

## How to Verify

1. Download `config/genesis_pulse.json` from the repository.
2. Compute the BLAKE3-512 hash:
   ```bash
   b3sum --raw --length 64 config/genesis_pulse.json | xxd -p -c 128
   ```
3. Compare against the fingerprint above and the `GENESIS_PULSE_HASH`
   constant in `src/lib.rs`.
4. Review the JSON parameters to confirm the 124M supply invariants.
