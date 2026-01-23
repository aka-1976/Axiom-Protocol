// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";
import "@openzeppelin/contracts/security/Pausable.sol";

/**
 * @title AxiomBridge
 * @dev Cross-chain bridge for Axiom Protocol
 * Supports lock/mint and burn/unlock mechanisms for cross-chain transfers
 */
contract AxiomBridge is ReentrancyGuard, Ownable, Pausable {
    
    // Events
    event TokensLocked(
        bytes32 indexed bridgeId,
        address indexed sender,
        uint256 amount,
        uint256 destinationChain,
        address recipient,
        uint256 timestamp
    );
    
    event TokensMinted(
        bytes32 indexed bridgeId,
        address indexed recipient,
        uint256 amount,
        uint256 timestamp
    );
    
    event TokensBurned(
        bytes32 indexed bridgeId,
        address indexed sender,
        uint256 amount,
        uint256 destinationChain,
        address recipient,
        uint256 timestamp
    );
    
    event TokensUnlocked(
        bytes32 indexed bridgeId,
        address indexed recipient,
        uint256 amount,
        uint256 timestamp
    );
    
    event OracleAdded(address indexed oracle);
    event OracleRemoved(address indexed oracle);
    
    // State variables
    mapping(bytes32 => bool) public processedBridges;
    mapping(address => bool) public oracles;
    mapping(bytes32 => uint256) public bridgeAmounts;
    
    uint256 public requiredOracles = 3;
    uint256 public minBridgeAmount = 1e9; // 1 AXM (9 decimals)
    uint256 public maxBridgeAmount = 1000000e9; // 1M AXM
    uint256 public totalLocked;
    
    WrappedAxiom public immutable wrappedToken;
    
    constructor(address _wrappedToken) {
        wrappedToken = WrappedAxiom(_wrappedToken);
    }
    
    /**
     * @dev Lock native tokens on Axiom chain to bridge to another chain
     * @param amount Amount of tokens to lock
     * @param destinationChain Target chain ID
     * @param recipient Recipient address on destination chain
     */
    function lockTokens(
        uint256 amount,
        uint256 destinationChain,
        address recipient
    ) external payable nonReentrant whenNotPaused returns (bytes32) {
        require(msg.value == amount, "Amount mismatch");
        require(amount >= minBridgeAmount, "Amount too small");
        require(amount <= maxBridgeAmount, "Amount too large");
        require(recipient != address(0), "Invalid recipient");
        
        bytes32 bridgeId = keccak256(
            abi.encodePacked(
                msg.sender,
                amount,
                destinationChain,
                recipient,
                block.timestamp,
                block.number
            )
        );
        
        require(!processedBridges[bridgeId], "Bridge ID already used");
        
        bridgeAmounts[bridgeId] = amount;
        totalLocked += amount;
        
        emit TokensLocked(
            bridgeId,
            msg.sender,
            amount,
            destinationChain,
            recipient,
            block.timestamp
        );
        
        return bridgeId;
    }
    
    /**
     * @dev Mint wrapped tokens on destination chain
     * @param bridgeId Unique bridge transaction ID
     * @param recipient Address to receive wrapped tokens
     * @param amount Amount to mint
     * @param signatures Oracle signatures (must have at least requiredOracles)
     */
    function mintWrapped(
        bytes32 bridgeId,
        address recipient,
        uint256 amount,
        bytes[] calldata signatures
    ) external nonReentrant whenNotPaused {
        require(!processedBridges[bridgeId], "Already processed");
        require(signatures.length >= requiredOracles, "Not enough signatures");
        require(recipient != address(0), "Invalid recipient");
        
        // Verify oracle signatures
        // In production, implement proper signature verification
        // For now, we trust the oracle network
        
        processedBridges[bridgeId] = true;
        wrappedToken.mint(recipient, amount);
        
        emit TokensMinted(bridgeId, recipient, amount, block.timestamp);
    }
    
    /**
     * @dev Burn wrapped tokens to bridge back to source chain
     * @param amount Amount of wrapped tokens to burn
     * @param destinationChain Target chain ID (usually Axiom)
     * @param recipient Recipient address on destination chain
     */
    function burnWrapped(
        uint256 amount,
        uint256 destinationChain,
        address recipient
    ) external nonReentrant whenNotPaused returns (bytes32) {
        require(amount >= minBridgeAmount, "Amount too small");
        require(amount <= maxBridgeAmount, "Amount too large");
        require(recipient != address(0), "Invalid recipient");
        
        // Burn wrapped tokens
        wrappedToken.burnFrom(msg.sender, amount);
        
        bytes32 bridgeId = keccak256(
            abi.encodePacked(
                msg.sender,
                amount,
                destinationChain,
                recipient,
                block.timestamp,
                block.number
            )
        );
        
        emit TokensBurned(
            bridgeId,
            msg.sender,
            amount,
            destinationChain,
            recipient,
            block.timestamp
        );
        
        return bridgeId;
    }
    
    /**
     * @dev Unlock native tokens on Axiom chain
     * @param bridgeId Unique bridge transaction ID
     * @param recipient Address to receive unlocked tokens
     * @param amount Amount to unlock
     * @param signatures Oracle signatures
     */
    function unlockTokens(
        bytes32 bridgeId,
        address recipient,
        uint256 amount,
        bytes[] calldata signatures
    ) external nonReentrant whenNotPaused {
        require(!processedBridges[bridgeId], "Already processed");
        require(signatures.length >= requiredOracles, "Not enough signatures");
        require(recipient != address(0), "Invalid recipient");
        require(address(this).balance >= amount, "Insufficient liquidity");
        
        processedBridges[bridgeId] = true;
        totalLocked -= amount;
        
        (bool success, ) = recipient.call{value: amount}("");
        require(success, "Transfer failed");
        
        emit TokensUnlocked(bridgeId, recipient, amount, block.timestamp);
    }
    
    // Oracle management
    function addOracle(address oracle) external onlyOwner {
        require(oracle != address(0), "Invalid oracle");
        require(!oracles[oracle], "Oracle already exists");
        oracles[oracle] = true;
        emit OracleAdded(oracle);
    }
    
    function removeOracle(address oracle) external onlyOwner {
        require(oracles[oracle], "Oracle does not exist");
        oracles[oracle] = false;
        emit OracleRemoved(oracle);
    }
    
    function setRequiredOracles(uint256 _required) external onlyOwner {
        require(_required > 0, "Must require at least 1 oracle");
        requiredOracles = _required;
    }
    
    function setMinBridgeAmount(uint256 _min) external onlyOwner {
        minBridgeAmount = _min;
    }
    
    function setMaxBridgeAmount(uint256 _max) external onlyOwner {
        maxBridgeAmount = _max;
    }
    
    // Emergency functions
    function pause() external onlyOwner {
        _pause();
    }
    
    function unpause() external onlyOwner {
        _unpause();
    }
    
    // Allow contract to receive ETH/native tokens
    receive() external payable {}
    
    // View functions
    function getBridgeStatus(bytes32 bridgeId) external view returns (bool processed, uint256 amount) {
        return (processedBridges[bridgeId], bridgeAmounts[bridgeId]);
    }
}

/**
 * @title WrappedAxiom
 * @dev ERC20 wrapped version of Axiom (AXM) for use on other chains
 */
contract WrappedAxiom is ERC20, Ownable {
    address public bridge;
    
    constructor() ERC20("Wrapped Axiom", "wAXM") {
        // Initial supply is 0 - minted only through bridge
    }
    
    function setBridge(address _bridge) external onlyOwner {
        require(_bridge != address(0), "Invalid bridge address");
        bridge = _bridge;
    }
    
    function mint(address to, uint256 amount) external {
        require(msg.sender == bridge, "Only bridge can mint");
        _mint(to, amount);
    }
    
    function burnFrom(address from, uint256 amount) public {
        require(msg.sender == bridge, "Only bridge can burn");
        _burn(from, amount);
    }
    
    function decimals() public pure override returns (uint8) {
        return 9; // Match Axiom's 9 decimals
    }
}
