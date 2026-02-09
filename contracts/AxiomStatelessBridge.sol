// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/**
 * @title IRiscZeroVerifier
 * @notice Minimal interface for verifying RISC Zero receipts on-chain.
 *
 * A production deployment would point to the official RISC Zero
 * `RiscZeroGroth16Verifier` contract deployed on Ethereum.
 */
interface IRiscZeroVerifier {
    /**
     * @notice Verify a Groth16-wrapped STARK receipt.
     * @param seal      The Groth16 proof bytes (SNARK wrapper).
     * @param imageId   The 256-bit image ID of the guest program.
     * @param journalDigest SHA-256 digest of the journal committed by the guest.
     */
    function verify(
        bytes calldata seal,
        bytes32 imageId,
        bytes32 journalDigest
    ) external view;
}

/**
 * @title AxiomStatelessBridge
 * @notice Stateless bridge that anchors Axiom pulse state on Ethereum
 *         using RISC Zero receipt verification.
 *
 * Instead of trusting oracle multi-sigs, this contract validates a
 * RISC-V STARK receipt (Groth16-wrapped) that proves the 124M supply
 * integrity was computed correctly inside the Axiom node's zkVM.
 *
 * If the receipt is valid, the contract updates `lastAnchoredPulse`
 * — giving any Ethereum dApp a trustless view of the Axiom chain.
 */
contract AxiomStatelessBridge {

    // ----------------------------------------------------------------
    // Events
    // ----------------------------------------------------------------

    event PulseAnchored(
        uint64 indexed height,
        bytes32 pulseHash,
        bytes32 journalDigest,
        uint64  timestamp
    );

    // ----------------------------------------------------------------
    // State
    // ----------------------------------------------------------------

    /// RISC Zero on-chain verifier (e.g., RiscZeroGroth16Verifier).
    IRiscZeroVerifier public immutable verifier;

    /// Image ID of the Axiom supply-integrity guest program.
    /// This is the deterministic fingerprint of `methods/guest/src/main.rs`.
    bytes32 public immutable guestImageId;

    /// Last successfully anchored pulse hash (256-bit truncation of the
    /// 512-bit BLAKE3 pulse hash for EVM compatibility).
    bytes32 public lastAnchoredPulse;

    /// Last anchored block height.
    uint64 public lastAnchoredHeight;

    /// Timestamp of the last anchored pulse.
    uint64 public lastAnchoredTimestamp;

    /// Owner for administrative functions.
    address public owner;

    // ----------------------------------------------------------------
    // Constructor
    // ----------------------------------------------------------------

    /**
     * @param _verifier    Address of the deployed IRiscZeroVerifier.
     * @param _guestImageId Image ID of the Axiom supply-integrity guest.
     */
    constructor(address _verifier, bytes32 _guestImageId) {
        require(_verifier != address(0), "AxiomStatelessBridge: zero verifier");
        verifier     = IRiscZeroVerifier(_verifier);
        guestImageId = _guestImageId;
        owner        = msg.sender;
    }

    // ----------------------------------------------------------------
    // Core: Receipt-Validated Anchor
    // ----------------------------------------------------------------

    /**
     * @notice Anchor a new Axiom pulse by submitting a RISC Zero receipt.
     *
     * @param seal           Groth16 proof bytes from the RISC Zero receipt.
     * @param journalDigest  SHA-256 digest of the receipt journal.
     * @param height         Axiom block height proven by this receipt.
     * @param pulseHash      256-bit pulse hash for the anchored state.
     * @param timestamp      Unix timestamp of the pulse.
     */
    function anchorWithReceipt(
        bytes   calldata seal,
        bytes32 journalDigest,
        uint64  height,
        bytes32 pulseHash,
        uint64  timestamp
    ) external {
        // Height must be strictly increasing.
        require(height > lastAnchoredHeight, "AxiomStatelessBridge: stale height");

        // Verify the RISC Zero receipt on-chain.
        // Reverts if the proof is invalid.
        verifier.verify(seal, guestImageId, journalDigest);

        // Update anchored state.
        lastAnchoredPulse     = pulseHash;
        lastAnchoredHeight    = height;
        lastAnchoredTimestamp  = timestamp;

        emit PulseAnchored(height, pulseHash, journalDigest, timestamp);
    }

    // ----------------------------------------------------------------
    // View helpers
    // ----------------------------------------------------------------

    /**
     * @notice Check whether a given height has already been anchored.
     */
    function isAnchored(uint64 height) external view returns (bool) {
        return height <= lastAnchoredHeight;
    }

    // ----------------------------------------------------------------
    // Admin
    // ----------------------------------------------------------------

    function transferOwnership(address newOwner) external {
        require(msg.sender == owner, "AxiomStatelessBridge: not owner");
        require(newOwner != address(0), "AxiomStatelessBridge: zero address");
        owner = newOwner;
    }
}
