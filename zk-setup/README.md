# AXIOM Protocol ZK-STARK Setup

This directory contains the setup and key management infrastructure for the AXIOM Protocol's ZK-STARK privacy features.

## 🔐 Overview

The AXIOM Protocol uses ZK-STARKs (Zero-Knowledge Scalable Transparent Arguments of Knowledge) to provide cryptographic privacy for transactions while maintaining network consensus. ZK-STARKs require no trusted setup ceremony — the proof system is fully transparent.

## 📋 Ceremony Process

### Phase 1: Trusted Setup Ceremony

The trusted setup generates two critical cryptographic keys:

1. **Proving Key** (`proving_key.bin`) - Large binary file (~50-200MB) used to generate proofs
2. **Verification Key** (`verification_key.json`) - Small JSON file used to verify proofs

### Running the Ceremony

```bash
# Make the ceremony script executable
chmod +x zk-setup/ceremony.sh

# Run the trusted setup ceremony
./zk-setup/ceremony.sh
```

This will:
- Generate both proving and verification keys
- Create ceremony transcripts and logs
- Output file hashes for verification
- Provide security reminders

### Ceremony Security

⚠️ **CRITICAL SECURITY NOTES:**

- The ceremony generates "toxic waste" (trapdoor information) that could compromise the entire network
- Toxic waste MUST be securely destroyed after key generation
- The ceremony should be performed on air-gapped hardware
- Multiple participants should verify the process
- Never reuse the same parameters for different circuits

## 🔑 Key Management

### Distribution Strategy

**Proving Key** (Large file - DO NOT commit to git):
- Upload to decentralized storage (IPFS, Arweave, Filecoin)
- Distribute via torrent or direct download
- Nodes download on first run and cache locally

**Verification Key** (Small file - CAN be committed):
- Commit `verification_key.json` to the repository
- Include in node releases
- Used for proof verification

### Node Key Handling

Nodes automatically download required keys on first startup:

```bash
# Download and verify ZK keys
./zk-setup/download-keys.sh
```

Keys are stored in:
- Linux/macOS: `~/.axiom/keys/`
- Windows: `%APPDATA%\axiom\keys\`

## 🔍 Verification

### Ceremony Transcript

After each ceremony, a transcript is generated containing:
- Ceremony ID and timestamps
- Circuit specifications
- Key file hashes (SHA256)
- Verification instructions
- Security attestations

### Key Verification

```bash
# Verify proving key hash
sha256sum ~/.axiom/keys/proving_key.bin

# Verify verification key
cat ~/.axiom/keys/verification_key.json
```

## 🛠️ Technical Details

### Cryptographic Parameters

- **Scheme**: Winterfell (ZK-STARK)
- **Field**: f128 (128-bit field elements)
- **Circuit**: AxiomTransactionCircuit
- **Constraints**: Balance verification, address derivation, amount validation

### Circuit Description

The ZK circuit proves:
1. **Address Derivation**: Secret key correctly derives public address
2. **Balance Verification**: Transaction amounts don't exceed available balance
3. **Amount Validation**: Transfer amounts and fees are correctly calculated

### Proof Generation

```rust
// Generate proof for a transaction
let proof = generate_transaction_proof(
    secret_key,
    current_balance,
    transfer_amount,
    fee
)?;
```

### Proof Verification

```rust
// Verify proof without revealing transaction details
let valid = verify_transaction_proof(
    proof,
    public_address,
    transfer_amount,
    fee
)?;
```

## 📚 Documentation

### For Developers

- **Circuit Design**: See `src/circuit.rs`
- **Proof Generation**: See `src/genesis.rs`
- **Integration**: See `src/main.rs` and `src/chain.rs`

### For Node Operators

- **Key Download**: Run `./zk-setup/download-keys.sh`
- **Verification**: Check key hashes against ceremony transcript
- **Updates**: Monitor repository for key updates

### For Auditors

- **Ceremony Logs**: Check `zk-setup/ceremony-logs/`
- **Transcripts**: Verify against published hashes
- **Source Code**: Review circuit implementation
- **Test Vectors**: Use provided test cases

## 🚨 Security Considerations

### Trusted Setup Risks

1. **Toxic Waste**: Could be used to forge proofs
2. **Single Point of Failure**: Compromised setup compromises entire network
3. **Ceremony Integrity**: Must be performed correctly and verifiably

### Mitigation Strategies

1. **Multi-Party Ceremony**: Multiple participants contribute randomness
2. **Transcript Verification**: Public verification of ceremony steps
3. **Toxic Waste Destruction**: Cryptographic erasure of trapdoors
4. **Open Source**: Transparent ceremony and verification code

### Future Improvements

- **Universal SRS**: Move to Plonk/Halo2 for transparent setup
- **Recursive Proofs**: Enable proof composition
- **Multi-Circuit Setup**: Support for different transaction types

## 📞 Support

For questions about ZK setup or key management:

1. Check ceremony transcripts in `zk-setup/ceremony-logs/`
2. Review this documentation
3. Open an issue for technical questions
4. Contact maintainers for security concerns

## 🔄 Updates

When new keys are generated:

1. Update `download-keys.sh` with new IPFS hashes
2. Commit new `verification_key.json`
3. Update ceremony transcripts
4. Announce key rotation to node operators
5. Provide migration instructions

---

**Status**: 🟢 Production - Trusted setup, key management, and proof verification are robust and production-ready.