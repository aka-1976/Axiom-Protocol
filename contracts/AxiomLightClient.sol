// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/**
 * @title AxiomLightClient
 * @notice Pulse-Anchor mechanism for the Axiom Protocol on Ethereum.
 *
 * Stores the latest 512-bit pulse hash from the Axiom P2P mesh and
 * enforces a strict chronological chain: every submitted pulse must
 * link its `prevPulseHash` to the on-chain `latestPulseHash`.
 * If the chain is broken, the EVM rejects the update.
 *
 * This allows any Ethereum contract or off-chain verifier to trust
 * the most recent state of the 124M-supply Axiom network without
 * running a full node.
 */
contract AxiomLightClient {

    // ----------------------------------------------------------------
    // Events
    // ----------------------------------------------------------------

    event PulseAnchored(
        uint64 indexed height,
        bytes32 pulseHashHigh,
        bytes32 pulseHashLow,
        uint64  timestamp
    );

    // ----------------------------------------------------------------
    // State
    // ----------------------------------------------------------------

    /// The 512-bit pulse hash is stored as two 256-bit halves.
    bytes32 public latestPulseHashHigh;
    bytes32 public latestPulseHashLow;

    /// Last anchored block height from the Axiom chain.
    uint64 public latestHeight;

    /// Timestamp of the last anchored pulse.
    uint64 public latestTimestamp;

    /// Authorised relayer (set once at construction; zero = permissionless).
    address public relayer;

    /// Owner for administrative actions.
    address public owner;

    // ----------------------------------------------------------------
    // Modifiers
    // ----------------------------------------------------------------

    modifier onlyRelayer() {
        require(
            relayer == address(0) || msg.sender == relayer,
            "AxiomLightClient: caller is not the relayer"
        );
        _;
    }

    modifier onlyOwner() {
        require(msg.sender == owner, "AxiomLightClient: caller is not the owner");
        _;
    }

    // ----------------------------------------------------------------
    // Constructor
    // ----------------------------------------------------------------

    /**
     * @param _relayer Authorised relayer address. Pass address(0) for
     *                 permissionless operation.
     * @param _genesisPulseHashHigh Upper 256 bits of the genesis pulse hash.
     * @param _genesisPulseHashLow  Lower 256 bits of the genesis pulse hash.
     */
    constructor(
        address _relayer,
        bytes32 _genesisPulseHashHigh,
        bytes32 _genesisPulseHashLow
    ) {
        owner   = msg.sender;
        relayer = _relayer;

        // Seed the chain with the genesis pulse.
        latestPulseHashHigh = _genesisPulseHashHigh;
        latestPulseHashLow  = _genesisPulseHashLow;
    }

    // ----------------------------------------------------------------
    // Core: Pulse Anchor
    // ----------------------------------------------------------------

    /**
     * @notice Anchor a new Axiom pulse on Ethereum.
     * @dev    The caller must supply the `prevPulseHash` that links back
     *         to the current on-chain head. If it does not match, the
     *         transaction reverts — enforcing a strict chronological chain.
     *
     * @param height            Axiom block height of this pulse.
     * @param pulseHashHigh     Upper 256 bits of the new pulse hash.
     * @param pulseHashLow      Lower 256 bits of the new pulse hash.
     * @param prevPulseHashHigh Upper 256 bits of the previous pulse hash.
     * @param prevPulseHashLow  Lower 256 bits of the previous pulse hash.
     * @param timestamp         Unix timestamp of the pulse.
     */
    function anchorPulse(
        uint64  height,
        bytes32 pulseHashHigh,
        bytes32 pulseHashLow,
        bytes32 prevPulseHashHigh,
        bytes32 prevPulseHashLow,
        uint64  timestamp
    ) external onlyRelayer {
        // Enforce chronological link: prev must match current head.
        require(
            prevPulseHashHigh == latestPulseHashHigh &&
            prevPulseHashLow  == latestPulseHashLow,
            "AxiomLightClient: prev_pulse_hash mismatch — chain is broken"
        );

        // Height must be strictly increasing.
        require(height > latestHeight, "AxiomLightClient: height must increase");

        // Update on-chain state.
        latestPulseHashHigh = pulseHashHigh;
        latestPulseHashLow  = pulseHashLow;
        latestHeight        = height;
        latestTimestamp      = timestamp;

        emit PulseAnchored(height, pulseHashHigh, pulseHashLow, timestamp);
    }

    // ----------------------------------------------------------------
    // View helpers
    // ----------------------------------------------------------------

    /**
     * @notice Return the full 512-bit latest pulse hash as a 64-byte blob.
     */
    function getLatestPulseHash() external view returns (bytes memory) {
        return abi.encodePacked(latestPulseHashHigh, latestPulseHashLow);
    }

    // ----------------------------------------------------------------
    // Admin
    // ----------------------------------------------------------------

    function setRelayer(address _relayer) external onlyOwner {
        relayer = _relayer;
    }

    function transferOwnership(address newOwner) external onlyOwner {
        require(newOwner != address(0), "AxiomLightClient: zero address");
        owner = newOwner;
    }
}
