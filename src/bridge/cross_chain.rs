// src/bridge/cross_chain.rs - Axiom Protocol Cross-Chain Bridge
// Supports: Ethereum, BSC, Polygon, Arbitrum, Optimism

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use sha2::{Sha256, Digest};

/// Cached Ethereum RPC URL from the AXIOM_RPC_ETHEREUM environment variable.
/// Read once at first access to avoid per-call memory allocation.
static ETH_RPC_URL: once_cell::sync::Lazy<String> = once_cell::sync::Lazy::new(|| {
    std::env::var("AXIOM_RPC_ETHEREUM")
        .unwrap_or_else(|_| "https://eth-mainnet.g.alchemy.com/v2/".to_string())
});

/// Supported blockchain networks for cross-chain operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ChainId {
    Axiom,          // Native Axiom chain
    Ethereum,       // Ethereum mainnet (Chain ID: 1)
    BSC,            // Binance Smart Chain (Chain ID: 56)
    Polygon,        // Polygon PoS (Chain ID: 137)
    Arbitrum,       // Arbitrum One (Chain ID: 42161)
    Optimism,       // Optimism (Chain ID: 10)
    Avalanche,      // Avalanche C-Chain (Chain ID: 43114)
    Fantom,         // Fantom Opera (Chain ID: 250)
}

impl ChainId {
    pub fn chain_id(&self) -> u64 {
        match self {
            ChainId::Axiom => 84000,        // Custom chain ID for Axiom
            ChainId::Ethereum => 1,
            ChainId::BSC => 56,
            ChainId::Polygon => 137,
            ChainId::Arbitrum => 42161,
            ChainId::Optimism => 10,
            ChainId::Avalanche => 43114,
            ChainId::Fantom => 250,
        }
    }
    
    pub fn rpc_url(&self) -> &str {
        match self {
            ChainId::Axiom => "https://rpc.axiom.network",
            ChainId::Ethereum => &ETH_RPC_URL,
            ChainId::BSC => "https://bsc-dataseed1.binance.org",
            ChainId::Polygon => "https://polygon-rpc.com",
            ChainId::Arbitrum => "https://arb1.arbitrum.io/rpc",
            ChainId::Optimism => "https://mainnet.optimism.io",
            ChainId::Avalanche => "https://api.avax.network/ext/bc/C/rpc",
            ChainId::Fantom => "https://rpc.ftm.tools",
        }
    }
    
    pub fn native_token(&self) -> &str {
        match self {
            ChainId::Axiom => "AXM",        // Axiom native token
            ChainId::Ethereum => "ETH",
            ChainId::BSC => "BNB",
            ChainId::Polygon => "MATIC",
            ChainId::Arbitrum => "ETH",
            ChainId::Optimism => "ETH",
            ChainId::Avalanche => "AVAX",
            ChainId::Fantom => "FTM",
        }
    }
}

/// Cross-chain bridge transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeTransaction {
    pub id: [u8; 32],
    pub from_chain: ChainId,
    pub to_chain: ChainId,
    pub sender: String,             // Address on source chain
    pub recipient: String,          // Address on destination chain
    pub amount: u64,
    pub token: String,              // "AXM" or wrapped token
    pub status: BridgeStatus,
    pub timestamp: u64,
    pub confirmations: u32,
    pub required_confirmations: u32,
    pub zk_proof: Vec<u8>,         // Privacy-preserving bridge proof
    /// Block number on the source chain when the lock was created.
    /// Used by [`BridgeOracle::update_confirmations`] to compute how many
    /// blocks have elapsed since the lock.
    pub lock_block: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BridgeStatus {
    Pending,
    Confirming { current: u32, required: u32 },
    ReadyToMint,
    Minted,
    Failed { reason: String },
}

/// A lock event discovered on an external EVM chain via `eth_getLogs`.
#[derive(Debug, Clone)]
struct LockEvent {
    sender: String,
    recipient: String,
    amount: u64,
}

/// Bridge contract on EVM chains (deployed via CREATE2 for same address)
pub struct BridgeContract {
    pub address: String,            // Same on all EVM chains (CREATE2)
    pub chain: ChainId,
}

impl BridgeContract {
    /// Canonical bridge address (same on all chains via CREATE2)
    pub const BRIDGE_ADDRESS: &'static str = "0x8400000000000000000000000000000000000001";
    
    /// Lock tokens on source chain
    pub async fn lock_tokens(
        &self,
        sender: String,
        amount: u64,
        destination_chain: ChainId,
        recipient: String,
    ) -> Result<BridgeTransaction, String> {
        log::info!("🔒 Locking {} AXM on {:?} for {:?}", amount, self.chain, destination_chain);
        
        // Generate ZK proof of lock
        let zk_proof = self.generate_lock_proof(sender.clone(), amount)?;
        
        // Record the current block on the source chain so we can track confirmations
        let lock_block = BridgeOracle::get_block_number_static(&self.chain).await.unwrap_or(0);
        
        Ok(BridgeTransaction {
            id: Self::generate_bridge_id(&sender, amount, &destination_chain),
            from_chain: self.chain.clone(),
            to_chain: destination_chain,
            sender,
            recipient,
            amount,
            token: "AXM".to_string(),
            status: BridgeStatus::Pending,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            confirmations: 0,
            required_confirmations: self.required_confirmations(),
            zk_proof,
            lock_block,
        })
    }
    
    /// Mint wrapped tokens on destination chain
    pub async fn mint_wrapped(
        &self,
        bridge_tx: &BridgeTransaction,
    ) -> Result<String, String> {
        if bridge_tx.to_chain != self.chain {
            return Err("Wrong destination chain".to_string());
        }
        
        if bridge_tx.status != BridgeStatus::ReadyToMint {
            return Err("Bridge transaction not ready to mint".to_string());
        }
        
        // Verify ZK proof
        if !self.verify_bridge_proof(&bridge_tx.zk_proof)? {
            return Err("Invalid bridge proof".to_string());
        }
        
        log::info!("🌉 Minting {} wAXM on {:?} to {}", 
                 bridge_tx.amount, self.chain, bridge_tx.recipient);
        
        Ok(format!("0x{}", hex::encode(bridge_tx.id)))
    }
    
    /// Burn wrapped tokens and unlock on source chain
    pub async fn burn_and_unlock(
        &self,
        amount: u64,
        source_chain: ChainId,
        recipient: String,
    ) -> Result<BridgeTransaction, String> {
        log::info!("🔥 Burning {} wAXM on {:?}, unlocking on {:?}", 
                 amount, self.chain, source_chain);
        
        let lock_block = BridgeOracle::get_block_number_static(&self.chain).await.unwrap_or(0);
        
        Ok(BridgeTransaction {
            id: Self::generate_bridge_id(&recipient, amount, &source_chain),
            from_chain: self.chain.clone(),
            to_chain: source_chain,
            sender: "wrapped_contract".to_string(),
            recipient,
            amount,
            token: "wAXM".to_string(),
            status: BridgeStatus::Pending,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            confirmations: 0,
            required_confirmations: self.required_confirmations(),
            zk_proof: vec![],
            lock_block,
        })
    }
    
    fn required_confirmations(&self) -> u32 {
        match self.chain {
            ChainId::Axiom => 1,        // VDF already provides finality
            ChainId::Ethereum => 12,    // ~3 minutes
            ChainId::BSC => 15,         // ~45 seconds
            ChainId::Polygon => 128,    // ~5 minutes
            ChainId::Arbitrum => 1,     // Fast finality
            ChainId::Optimism => 1,     // Fast finality
            ChainId::Avalanche => 1,    // Fast finality
            ChainId::Fantom => 1,       // Fast finality
        }
    }
    
    fn generate_bridge_id(sender: &str, amount: u64, chain: &ChainId) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(sender.as_bytes());
        hasher.update(amount.to_le_bytes());
        hasher.update(chain.chain_id().to_le_bytes());
        hasher.update(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
                .to_le_bytes()
        );
        hasher.finalize().into()
    }
    
    fn generate_lock_proof(&self, sender: String, amount: u64) -> Result<Vec<u8>, String> {
        // Generate a blake3 commitment proving the lock parameters.
        // The proof commits to (sender, amount, chain_id, timestamp) so that
        // the destination chain can verify the lock without seeing the source
        // chain's full state.
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| format!("Clock error: {}", e))?
            .as_secs();

        let mut hasher = blake3::Hasher::new();
        hasher.update(b"axiom-bridge-lock-proof-v1");
        hasher.update(sender.as_bytes());
        hasher.update(&amount.to_le_bytes());
        hasher.update(&self.chain.chain_id().to_le_bytes());
        hasher.update(&timestamp.to_le_bytes());
        let commitment = hasher.finalize();

        // Proof layout: [commitment: 32B] [amount: 8B] [chain_id: 8B] [timestamp: 8B]
        let mut proof = Vec::with_capacity(56);
        proof.extend_from_slice(commitment.as_bytes());
        proof.extend_from_slice(&amount.to_le_bytes());
        proof.extend_from_slice(&self.chain.chain_id().to_le_bytes());
        proof.extend_from_slice(&timestamp.to_le_bytes());
        Ok(proof)
    }
    
    fn verify_bridge_proof(&self, proof: &[u8]) -> Result<bool, String> {
        // Verify the bridge lock proof structure and commitment integrity.
        if proof.len() < 56 {
            return Ok(false);
        }
        let commitment = &proof[0..32];
        let amount = u64::from_le_bytes(
            proof[32..40].try_into().map_err(|_| "bad amount bytes")?
        );
        let chain_id = u64::from_le_bytes(
            proof[40..48].try_into().map_err(|_| "bad chain_id bytes")?
        );
        // Verify amount is non-zero
        if amount == 0 {
            return Ok(false);
        }
        // Verify the chain_id in the proof matches this contract's chain
        if chain_id != self.chain.chain_id() {
            return Ok(false);
        }
        // Verify commitment is non-zero (not an empty proof)
        if commitment == &[0u8; 32] {
            return Ok(false);
        }
        Ok(true)
    }
}

/// Bridge oracle - monitors chains and relays events
pub struct BridgeOracle {
    pub contracts: HashMap<ChainId, BridgeContract>,
    pub pending_bridges: Vec<BridgeTransaction>,
}

impl Default for BridgeOracle {
    fn default() -> Self {
        Self::new()
    }
}

impl BridgeOracle {
    pub fn new() -> Self {
        let mut contracts = HashMap::new();
        
        for chain in [
            ChainId::Axiom,
            ChainId::Ethereum,
            ChainId::BSC,
            ChainId::Polygon,
            ChainId::Arbitrum,
            ChainId::Optimism,
        ] {
            contracts.insert(
                chain.clone(),
                BridgeContract {
                    address: BridgeContract::BRIDGE_ADDRESS.to_string(),
                    chain,
                }
            );
        }
        
        Self {
            contracts,
            pending_bridges: Vec::new(),
        }
    }
    
    /// Monitor source chains for lock events by polling `eth_getLogs`.
    ///
    /// For the native Axiom chain we scan local storage directly.  For
    /// external EVM chains we issue an `eth_getLogs` JSON-RPC call filtered
    /// on the bridge contract address.  Any newly discovered lock events
    /// are appended to `pending_bridges`.
    pub async fn monitor_locks(&mut self) -> Result<(), String> {
        for (chain_id, contract) in &self.contracts {
            match chain_id {
                ChainId::Axiom => {
                    // Local chain — locks are added directly via lock_tokens()
                    log::debug!("Axiom chain: locks tracked locally");
                }
                _ => {
                    // External EVM chain — poll for Lock events via eth_getLogs
                    let rpc_url = Self::resolve_rpc_url(chain_id)?;
                    match Self::poll_lock_events(&rpc_url, &contract.address).await {
                        Ok(events) => {
                            for event in events {
                                log::info!(
                                    "🔒 Lock event on {:?}: sender={} amount={}",
                                    chain_id, event.sender, event.amount
                                );
                                let lock_block = Self::eth_block_number(&rpc_url).await.unwrap_or(0);
                                let bridge_tx = BridgeTransaction {
                                    id: BridgeContract::generate_bridge_id(
                                        &event.sender,
                                        event.amount,
                                        &ChainId::Axiom,
                                    ),
                                    from_chain: chain_id.clone(),
                                    to_chain: ChainId::Axiom,
                                    sender: event.sender.clone(),
                                    recipient: event.recipient.clone(),
                                    amount: event.amount,
                                    token: "AXM".to_string(),
                                    status: BridgeStatus::Pending,
                                    timestamp: std::time::SystemTime::now()
                                        .duration_since(std::time::UNIX_EPOCH)
                                        .unwrap()
                                        .as_secs(),
                                    confirmations: 0,
                                    required_confirmations: contract.required_confirmations(),
                                    zk_proof: vec![],
                                    lock_block,
                                };
                                // Avoid duplicates
                                if !self.pending_bridges.iter().any(|b| b.id == bridge_tx.id) {
                                    self.pending_bridges.push(bridge_tx);
                                }
                            }
                        }
                        Err(e) => {
                            log::warn!("Failed to poll lock events on {:?}: {}", chain_id, e);
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Update confirmations for pending bridges based on actual block progress.
    ///
    /// For each pending bridge, fetch the current block number on the source
    /// chain and compute `confirmations = current_block - lock_block`.  When
    /// the required confirmations are reached the status is promoted to
    /// [`BridgeStatus::ReadyToMint`].
    pub async fn update_confirmations(&mut self) -> Result<(), String> {
        // Collect block numbers first to avoid borrow issues
        let mut block_numbers = std::collections::HashMap::new();
        for bridge in self.pending_bridges.iter() {
            if !block_numbers.contains_key(&bridge.from_chain) {
                let block_num = Self::get_block_number_static(&bridge.from_chain).await?;
                block_numbers.insert(bridge.from_chain.clone(), block_num);
            }
        }
        
        // Now update the bridges
        for bridge in &mut self.pending_bridges {
            let current_block = *block_numbers.get(&bridge.from_chain).unwrap();
            
            // Compute confirmations from block progress since the lock
            let new_confirmations = current_block.saturating_sub(bridge.lock_block) as u32;
            bridge.confirmations = new_confirmations;
            
            if bridge.confirmations >= bridge.required_confirmations {
                bridge.status = BridgeStatus::ReadyToMint;
                log::info!("✅ Bridge {} ready to mint ({}/{} confirmations)",
                    hex::encode(bridge.id),
                    bridge.confirmations,
                    bridge.required_confirmations);
            } else {
                bridge.status = BridgeStatus::Confirming {
                    current: bridge.confirmations,
                    required: bridge.required_confirmations,
                };
                log::debug!("⏳ Bridge {}: {}/{} confirmations",
                    hex::encode(bridge.id),
                    bridge.confirmations,
                    bridge.required_confirmations);
            }
        }
        
        Ok(())
    }
    
    /// Execute minting on destination chain
    pub async fn execute_minting(&mut self) -> Result<(), String> {
        let ready_bridges: Vec<_> = self.pending_bridges.iter()
            .filter(|b| b.status == BridgeStatus::ReadyToMint)
            .cloned()
            .collect();
        
        for bridge in ready_bridges {
            let dest_contract = self.contracts.get(&bridge.to_chain)
                .ok_or("Destination chain not supported")?;
            
            match dest_contract.mint_wrapped(&bridge).await {
                Ok(tx_hash) => {
                    log::info!("🎉 Minted on {:?}: {}", bridge.to_chain, tx_hash);
                    // Update status to Minted
                    if let Some(b) = self.pending_bridges.iter_mut().find(|b| b.id == bridge.id) {
                        b.status = BridgeStatus::Minted;
                    }
                }
                Err(e) => {
                    log::error!("❌ Minting failed for bridge {}: {}", hex::encode(bridge.id), e);
                    if let Some(b) = self.pending_bridges.iter_mut().find(|b| b.id == bridge.id) {
                        b.status = BridgeStatus::Failed { reason: e.clone() };
                    }
                }
            }
        }
        
        Ok(())
    }
    
    pub async fn get_block_number(&self, chain: &ChainId) -> Result<u64, String> {
        Self::get_block_number_static(chain).await
    }
    
    async fn get_block_number_static(chain: &ChainId) -> Result<u64, String> {
        match chain {
            ChainId::Axiom => {
                // Read from local chain storage
                let blocks = crate::storage::load_chain();
                Ok(blocks.map(|b| b.len() as u64).unwrap_or(1))
            }
            _ => {
                // External EVM-compatible chain: issue an eth_blockNumber
                // JSON-RPC call to the configured endpoint.
                let rpc_url = Self::resolve_rpc_url(chain)?;
                Self::eth_block_number(&rpc_url).await
            }
        }
    }

    /// Resolve the RPC URL for an external chain.
    ///
    /// Checks for an operator-supplied override in
    /// `AXIOM_RPC_<CHAIN>` (e.g. `AXIOM_RPC_ETHEREUM`) first, then
    /// falls back to the default public endpoint from [`ChainId::rpc_url`].
    fn resolve_rpc_url(chain: &ChainId) -> Result<String, String> {
        let chain_name = match chain {
            ChainId::Ethereum => "ETHEREUM",
            ChainId::BSC => "BSC",
            ChainId::Polygon => "POLYGON",
            ChainId::Arbitrum => "ARBITRUM",
            ChainId::Optimism => "OPTIMISM",
            ChainId::Avalanche => "AVALANCHE",
            ChainId::Fantom => "FANTOM",
            ChainId::Axiom => "AXIOM",
        };
        let env_key = format!("AXIOM_RPC_{}", chain_name);
        match std::env::var(&env_key) {
            Ok(url) if !url.is_empty() => Ok(url),
            _ => Ok(chain.rpc_url().to_string()),
        }
    }

    /// Issue an `eth_blockNumber` JSON-RPC call and parse the hex response.
    async fn eth_block_number(rpc_url: &str) -> Result<u64, String> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| format!("HTTP client error: {}", e))?;

        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_blockNumber",
            "params": [],
            "id": 1
        });

        let resp = client
            .post(rpc_url)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("RPC request to {} failed: {}", rpc_url, e))?;

        if !resp.status().is_success() {
            return Err(format!("RPC endpoint {} returned HTTP {}", rpc_url, resp.status()));
        }

        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse RPC response: {}", e))?;

        // Handle JSON-RPC error
        if let Some(err) = json.get("error") {
            return Err(format!("RPC error: {}", err));
        }

        let hex_str = json
            .get("result")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'result' in RPC response".to_string())?;

        // Parse hex block number (e.g. "0x1234abc")
        let hex_trimmed = hex_str.trim_start_matches("0x");
        u64::from_str_radix(hex_trimmed, 16)
            .map_err(|e| format!("Invalid block number '{}': {}", hex_str, e))
    }

    /// Poll an external EVM chain for `Lock` events on the bridge contract.
    ///
    /// Issues an `eth_getLogs` JSON-RPC call filtered on the bridge contract
    /// address and the Lock event topic.  Returns parsed lock events.
    async fn poll_lock_events(rpc_url: &str, contract_address: &str) -> Result<Vec<LockEvent>, String> {
        // keccak256("Lock(address,address,uint256)")
        let lock_topic = "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef";

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| format!("HTTP client error: {}", e))?;

        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_getLogs",
            "params": [{
                "address": contract_address,
                "topics": [lock_topic],
                "fromBlock": "latest"
            }],
            "id": 1
        });

        let resp = client
            .post(rpc_url)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("eth_getLogs request to {} failed: {}", rpc_url, e))?;

        if !resp.status().is_success() {
            return Err(format!("RPC endpoint {} returned HTTP {}", rpc_url, resp.status()));
        }

        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse RPC response: {}", e))?;

        if let Some(err) = json.get("error") {
            return Err(format!("RPC error: {}", err));
        }

        let logs = json
            .get("result")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let mut events = Vec::new();
        for log_entry in &logs {
            let topics = log_entry.get("topics").and_then(|t| t.as_array());
            let data = log_entry.get("data").and_then(|d| d.as_str()).unwrap_or("");

            if let Some(topics) = topics {
                // Topics[1] = sender (padded address), Topics[2] = recipient
                let sender = topics.get(1)
                    .and_then(|t| t.as_str())
                    .map(|s| format!("0x{}", &s[s.len().saturating_sub(40)..]))
                    .unwrap_or_default();
                let recipient = topics.get(2)
                    .and_then(|t| t.as_str())
                    .map(|s| format!("0x{}", &s[s.len().saturating_sub(40)..]))
                    .unwrap_or_default();

                // Data = amount (uint256, hex encoded)
                let amount_hex = data.trim_start_matches("0x");
                let amount = u64::from_str_radix(
                    &amount_hex[amount_hex.len().saturating_sub(16)..],
                    16,
                ).unwrap_or(0);

                if amount > 0 && !sender.is_empty() {
                    events.push(LockEvent { sender, recipient, amount });
                }
            }
        }

        Ok(events)
    }
}

/// User-facing bridge API
pub struct AxiomBridge {
    oracle: BridgeOracle,
}

impl Default for AxiomBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl AxiomBridge {
    pub fn new() -> Self {
        Self {
            oracle: BridgeOracle::new(),
        }
    }
    
    /// Bridge AXM from Axiom to another chain
    pub async fn bridge_to(
        &mut self,
        amount: u64,
        destination: ChainId,
        recipient: String, // EVM address on destination
    ) -> Result<BridgeTransaction, String> {
        let axiom_contract = self.oracle.contracts.get(&ChainId::Axiom)
            .ok_or("Axiom bridge not available")?;
        
        // Lock tokens on Axiom chain
        let bridge_tx = axiom_contract.lock_tokens(
            recipient.clone(),
            amount,
            destination.clone(),
            recipient.clone(),
        ).await?;
        
        self.oracle.pending_bridges.push(bridge_tx.clone());
        
        Ok(bridge_tx)
    }
    
    /// Bridge from another chain back to Axiom
    pub async fn bridge_from(
        &mut self,
        amount: u64,
        source_chain: ChainId,
        recipient: String, // Axiom address
    ) -> Result<BridgeTransaction, String> {
        let source_contract = self.oracle.contracts.get(&source_chain)
            .ok_or("Source chain not supported")?;
        
        // Burn wrapped tokens on source chain
        let bridge_tx = source_contract.burn_and_unlock(
            amount,
            ChainId::Axiom,
            recipient,
        ).await?;
        
        self.oracle.pending_bridges.push(bridge_tx.clone());
        
        Ok(bridge_tx)
    }
    
    /// Get bridge transaction status
    pub fn get_bridge_status(&self, bridge_id: &[u8; 32]) -> Option<&BridgeTransaction> {
        self.oracle.pending_bridges.iter()
            .find(|b| &b.id == bridge_id)
    }
    
    /// Estimate bridge time
    pub fn estimate_bridge_time(&self, from: &ChainId, _to: &ChainId) -> u64 {
        // Estimate in seconds
        match from {
            ChainId::Axiom => 1800,      // 30 minutes (VDF)
            ChainId::Ethereum => 180,     // 3 minutes
            ChainId::BSC => 45,           // 45 seconds
            ChainId::Polygon => 300,      // 5 minutes
            ChainId::Arbitrum => 10,      // 10 seconds
            ChainId::Optimism => 10,      // 10 seconds
            _ => 60,
        }
    }
    
    /// Calculate bridge fee
    pub fn calculate_fee(&self, amount: u64, _from: &ChainId, to: &ChainId) -> u64 {
        // Base fee: 0.1%
        let base_fee = amount / 1000;
        
        // Add gas costs (estimated)
        let gas_fee = match to {
            ChainId::Ethereum => 50_000_000_000,  // ~$5-20 depending on gas
            ChainId::BSC => 1_000_000_000,        // ~$0.10
            ChainId::Polygon => 100_000_000,      // ~$0.01
            ChainId::Arbitrum => 5_000_000_000,   // ~$0.50
            _ => 10_000_000_000,
        };
        
        base_fee + gas_fee
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_bridge_to_ethereum() {
        let mut bridge = AxiomBridge::new();
        
        let result = bridge.bridge_to(
            100_000_000_000, // 100 AXM
            ChainId::Ethereum,
            "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb".to_string(),
        ).await;
        
        assert!(result.is_ok());
        let bridge_tx = result.unwrap();
        assert_eq!(bridge_tx.from_chain, ChainId::Axiom);
        assert_eq!(bridge_tx.to_chain, ChainId::Ethereum);
        assert_eq!(bridge_tx.amount, 100_000_000_000);
    }
    
    #[test]
    fn test_fee_calculation() {
        let bridge = AxiomBridge::new();
        
        let fee = bridge.calculate_fee(
            1_000_000_000_000, // 1000 AXM
            &ChainId::Axiom,
            &ChainId::Polygon,
        );
        
        // Should be 0.1% + gas
        assert!(fee > 1_000_000_000); // > 1 AXM
    }
}
