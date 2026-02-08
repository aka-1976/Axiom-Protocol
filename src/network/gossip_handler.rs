use bincode::{deserialize, serialize};
use libp2p::gossipsub::{IdentTopic, TopicHash};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info, trace, warn};

use crate::network::peer_manager::PeerManager;

/// Maximum message size (2MB)
const MAX_MESSAGE_SIZE: usize = 2 * 1024 * 1024;

/// Message deduplication cache TTL
const MESSAGE_CACHE_TTL: Duration = Duration::from_secs(300); // 5 minutes

/// Block message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockMessage {
    pub height: u64,
    pub hash: [u8; 32],
    pub prev_hash: [u8; 32],
    pub timestamp: u64,
    pub vdf_proof: Vec<u8>,
    pub nonce: u64,
    pub transaction_hashes: Vec<[u8; 32]>,
    pub miner: [u8; 32],
    pub reward: u64,
    pub full_block: Option<Vec<u8>>,
}

impl BlockMessage {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.height == 0 {
            return Err(ValidationError::InvalidHeight);
        }
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        if self.timestamp > now + 3600 {
            return Err(ValidationError::FutureTimestamp);
        }
        
        if self.transaction_hashes.len() > 1000 {
            return Err(ValidationError::TooManyTransactions);
        }
        
        if self.vdf_proof.is_empty() {
            return Err(ValidationError::MissingVdfProof);
        }
        
        Ok(())
    }
    
    pub fn hash(&self) -> [u8; 32] {
        let mut hasher = Sha3_256::new();
        hasher.update(&self.height.to_le_bytes());
        hasher.update(&self.hash);
        hasher.update(&self.prev_hash);
        hasher.finalize().into()
    }
}

/// Transaction message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionMessage {
    pub hash: [u8; 32],
    pub sender: [u8; 32],
    pub recipient: [u8; 32],
    pub amount: u64,
    pub fee: u64,
    pub nonce: u64,
    pub signature: Vec<u8>,
    pub zk_proof: Vec<u8>,
    pub commitment: Vec<u8>,
    pub timestamp: u64,
    pub full_tx: Option<Vec<u8>>,
}

impl TransactionMessage {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.signature.len() != 64 {
            return Err(ValidationError::InvalidSignature);
        }
        
        if self.zk_proof.is_empty() {
            return Err(ValidationError::MissingZkProof);
        }
        
        if self.amount == 0 {
            return Err(ValidationError::ZeroAmount);
        }
        
        if self.fee > 1_000_000_000_000 {
            return Err(ValidationError::ExcessiveFee);
        }
        
        if self.commitment.is_empty() {
            return Err(ValidationError::MissingCommitment);
        }
        
        Ok(())
    }
    
    pub fn hash(&self) -> [u8; 32] {
        let mut hasher = Sha3_256::new();
        hasher.update(&self.hash);
        hasher.update(&self.sender);
        hasher.update(&self.recipient);
        hasher.update(&self.nonce.to_le_bytes());
        hasher.finalize().into()
    }
}

/// Sync request message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncMessage {
    pub start_height: u64,
    pub end_height: u64,
    pub known_hashes: Vec<[u8; 32]>,
    pub request_type: SyncRequestType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncRequestType {
    Headers,
    FullBlocks,
    StateSnapshot,
}

/// Heartbeat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatMessage {
    pub height: u64,
    pub peer_count: usize,
    pub uptime: u64,
    pub timestamp: u64,
}

/// Gossip message envelope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GossipMessage {
    Block(BlockMessage),
    Transaction(TransactionMessage),
    Sync(SyncMessage),
    Heartbeat(HeartbeatMessage),
}

impl GossipMessage {
    pub fn encode(&self) -> Result<Vec<u8>, GossipError> {
        serialize(self).map_err(|e| GossipError::EncodeFailed(e.to_string()))
    }
    
    pub fn decode(data: &[u8]) -> Result<Self, GossipError> {
        if data.len() > MAX_MESSAGE_SIZE {
            return Err(GossipError::MessageTooLarge);
        }
        
        deserialize(data).map_err(|e| GossipError::DecodeFailed(e.to_string()))
    }
    
    pub fn message_type(&self) -> &str {
        match self {
            GossipMessage::Block(_) => "Block",
            GossipMessage::Transaction(_) => "Transaction",
            GossipMessage::Sync(_) => "Sync",
            GossipMessage::Heartbeat(_) => "Heartbeat",
        }
    }
}

/// Processed message for output
#[derive(Debug, Clone)]
pub struct ProcessedMessage {
    pub message: GossipMessage,
    pub source: String,
    pub received_at: Instant,
}

/// Gossip message handler
pub struct GossipHandler {
    message_cache: Arc<RwLock<HashMap<[u8; 32], Instant>>>,
    peer_manager: Arc<PeerManager>,
    block_tx: mpsc::UnboundedSender<ProcessedMessage>,
    transaction_tx: mpsc::UnboundedSender<ProcessedMessage>,
    sync_tx: mpsc::UnboundedSender<ProcessedMessage>,
    metrics: Arc<RwLock<GossipMetrics>>,
    subscribed_topics: HashSet<TopicHash>,
    processed_messages: HashSet<String>,
}

#[derive(Debug, Clone, Default)]
pub struct GossipMetrics {
    pub total_messages_received: u64,
    pub blocks_received: u64,
    pub transactions_received: u64,
    pub sync_requests_received: u64,
    pub invalid_messages: u64,
    pub duplicate_messages: u64,
}

impl GossipHandler {
    pub fn new(
        peer_manager: Arc<PeerManager>,
    ) -> (
        Self,
        mpsc::UnboundedReceiver<ProcessedMessage>,
        mpsc::UnboundedReceiver<ProcessedMessage>,
        mpsc::UnboundedReceiver<ProcessedMessage>,
    ) {
        let (block_tx, block_rx) = mpsc::unbounded_channel();
        let (transaction_tx, transaction_rx) = mpsc::unbounded_channel();
        let (sync_tx, sync_rx) = mpsc::unbounded_channel();
        
        let handler = Self {
            message_cache: Arc::new(RwLock::new(HashMap::new())),
            peer_manager,
            block_tx,
            transaction_tx,
            sync_tx,
            metrics: Arc::new(RwLock::new(GossipMetrics::default())),
            subscribed_topics: HashSet::new(),
            processed_messages: HashSet::new(),
        };
        
        // Start cache cleanup loop
        handler.start_cache_cleanup();
        
        (handler, block_rx, transaction_rx, sync_rx)
    }
    
    pub fn subscribe_topic(&mut self, topic: &IdentTopic) {
        self.subscribed_topics.insert(topic.hash());
    }
    
    pub fn is_subscribed(&self, topic_hash: &TopicHash) -> bool {
        self.subscribed_topics.contains(topic_hash)
    }
    
    pub fn mark_processed(&mut self, message_id: String) -> bool {
        self.processed_messages.insert(message_id)
    }
    
    pub fn was_processed(&self, message_id: &str) -> bool {
        self.processed_messages.contains(message_id)
    }
    
    pub async fn handle_message(
        &self,
        source: String,
        data: Vec<u8>,
        _topic: String,
    ) -> Result<(), GossipError> {
        let mut m = self.metrics.write().await;
        m.total_messages_received += 1;
        drop(m);
        
        if data.len() > MAX_MESSAGE_SIZE {
            warn!("Oversized message from {}: {} bytes", source, data.len());
            return Err(GossipError::MessageTooLarge);
        }
        
        let message = match GossipMessage::decode(&data) {
            Ok(msg) => msg,
            Err(e) => {
                warn!("Failed to decode message from {}: {}", source, e);
                let mut m = self.metrics.write().await;
                m.invalid_messages += 1;
                return Err(e);
            }
        };
        
        debug!("📨 Processing {} message from {}", message.message_type(), source);
        
        match &message {
            GossipMessage::Block(block) => {
                if let Err(e) = block.validate() {
                    warn!("Invalid block from {}: {:?}", source, e);
                    return Err(GossipError::Validation(e));
                }
                
                let mut m = self.metrics.write().await;
                m.blocks_received += 1;
                drop(m);
                
                self.block_tx.send(ProcessedMessage {
                    message: GossipMessage::Block(block.clone()),
                    source,
                    received_at: Instant::now(),
                }).map_err(|_| GossipError::ChannelClosed)?;
            }
            
            GossipMessage::Transaction(tx) => {
                if let Err(e) = tx.validate() {
                    warn!("Invalid transaction from {}: {:?}", source, e);
                    return Err(GossipError::Validation(e));
                }
                
                let mut m = self.metrics.write().await;
                m.transactions_received += 1;
                drop(m);
                
                self.transaction_tx.send(ProcessedMessage {
                    message: GossipMessage::Transaction(tx.clone()),
                    source,
                    received_at: Instant::now(),
                }).map_err(|_| GossipError::ChannelClosed)?;
            }
            
            GossipMessage::Sync(sync) => {
                if sync.end_height < sync.start_height {
                    warn!("Invalid sync range from {}", source);
                    return Err(GossipError::InvalidSyncRange);
                }
                
                let mut m = self.metrics.write().await;
                m.sync_requests_received += 1;
                drop(m);
                
                self.sync_tx.send(ProcessedMessage {
                    message: GossipMessage::Sync(sync.clone()),
                    source,
                    received_at: Instant::now(),
                }).map_err(|_| GossipError::ChannelClosed)?;
            }
            
            GossipMessage::Heartbeat(_) => {
                // Just log it
                trace!("Heartbeat received from {}", source);
            }
        }
        
        Ok(())
    }
    
    fn start_cache_cleanup(&self) {
        let cache = self.message_cache.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            
            loop {
                interval.tick().await;
                
                let now = Instant::now();
                cache.write().await.retain(|_, received_at| {
                    now.duration_since(*received_at) < MESSAGE_CACHE_TTL
                });
            }
        });
    }
    
    pub fn prepare_block_broadcast(&self, block: BlockMessage) -> Result<Vec<u8>, GossipError> {
        let message = GossipMessage::Block(block);
        message.encode()
    }
    
    pub fn prepare_transaction_broadcast(&self, tx: TransactionMessage) -> Result<Vec<u8>, GossipError> {
        let message = GossipMessage::Transaction(tx);
        message.encode()
    }
    
    pub fn prepare_heartbeat_broadcast(&self, heartbeat: HeartbeatMessage) -> Result<Vec<u8>, GossipError> {
        let message = GossipMessage::Heartbeat(heartbeat);
        message.encode()
    }
    
    pub async fn metrics(&self) -> GossipMetrics {
        self.metrics.read().await.clone()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GossipError {
    #[error("Message encoding failed: {0}")]
    EncodeFailed(String),
    
    #[error("Message decoding failed: {0}")]
    DecodeFailed(String),
    
    #[error("Message too large (max 2MB)")]
    MessageTooLarge,
    
    #[error("Validation error: {0:?}")]
    Validation(ValidationError),
    
    #[error("Invalid sync range")]
    InvalidSyncRange,
    
    #[error("Channel closed")]
    ChannelClosed,
}

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Invalid block height")]
    InvalidHeight,
    
    #[error("Future timestamp")]
    FutureTimestamp,
    
    #[error("Too many transactions")]
    TooManyTransactions,
    
    #[error("Missing VDF proof")]
    MissingVdfProof,
    
    #[error("Invalid signature")]
    InvalidSignature,
    
    #[error("Missing ZK proof")]
    MissingZkProof,
    
    #[error("Zero amount")]
    ZeroAmount,
    
    #[error("Excessive fee")]
    ExcessiveFee,
    
    #[error("Missing commitment")]
    MissingCommitment,
}
