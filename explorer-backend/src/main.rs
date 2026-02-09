use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use actix_cors::Cors;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

/// Block data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Block {
    index: u64,
    hash: String,
    previous_hash: String,
    timestamp: u64,
    transactions: Vec<Transaction>,
    miner: String,
    difficulty: u32,
    nonce: u64,
    merkle_root: String,
    vdf_output: Option<String>,
    vdf_proof: Option<String>,
    size: usize,
    reward: u64,
}

/// Transaction data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Transaction {
    hash: String,
    sender: String,
    recipient: String,
    amount: u64,
    fee: u64,
    timestamp: u64,
    signature: String,
    block_hash: Option<String>,
    block_index: Option<u64>,
    confirmations: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    zk_proof: Option<String>,
}

/// Network statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
struct NetworkStats {
    height: u64,
    total_transactions: u64,
    total_supply: u64,
    circulating_supply: u64,
    difficulty: u32,
    hash_rate: f64,
    peers: u32,
    mempool_size: u32,
    average_block_time: f64,
    latest_blocks: Vec<BlockSummary>,
}

/// Block summary for list views
#[derive(Debug, Clone, Serialize, Deserialize)]
struct BlockSummary {
    index: u64,
    hash: String,
    timestamp: u64,
    tx_count: usize,
    miner: String,
    reward: u64,
}

/// Address information
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AddressInfo {
    address: String,
    balance: u64,
    total_received: u64,
    total_sent: u64,
    tx_count: u32,
    recent_transactions: Vec<Transaction>,
}

/// Search result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
enum SearchResult {
    Block { data: Block },
    Transaction { data: Transaction },
    Address { data: AddressInfo },
    NotFound,
}

/// Shared application state
struct AppState {
    blocks: Mutex<Vec<Block>>,
    transactions: Mutex<Vec<Transaction>>,
}

impl AppState {
    fn new() -> Self {
        // Initialize with only the genesis block — real blocks arrive via chain sync
        let genesis_block = Block {
            index: 0,
            hash: "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            previous_hash: "0".repeat(64),
            timestamp: 1600000000,
            transactions: vec![],
            miner: "Genesis".to_string(),
            difficulty: 1,
            nonce: 0,
            merkle_root: "0".repeat(64),
            vdf_output: None,
            vdf_proof: None,
            size: 285,
            reward: 5000000000, // 50 AXM
        };

        Self {
            blocks: Mutex::new(vec![genesis_block]),
            transactions: Mutex::new(vec![]),
        }
    }
}

/// Get network statistics
async fn get_stats(data: web::Data<AppState>) -> impl Responder {
    let blocks = data.blocks.lock().unwrap();
    let transactions = data.transactions.lock().unwrap();
    
    let height = blocks.len() as u64 - 1;
    let latest_blocks: Vec<BlockSummary> = blocks.iter()
        .rev()
        .take(10)
        .map(|b| BlockSummary {
            index: b.index,
            hash: b.hash.clone(),
            timestamp: b.timestamp,
            tx_count: b.transactions.len(),
            miner: b.miner.clone(),
            reward: b.reward,
        })
        .collect();

    let stats = NetworkStats {
        height,
        total_transactions: transactions.len() as u64,
        total_supply: 124000000_00000000, // 124M AXM in satoshis
        circulating_supply: height * 5000000000, // 50 AXM per block
        difficulty: blocks.last().map(|b| b.difficulty).unwrap_or(1000),
        hash_rate: 0.0, // Computed when mining telemetry is available
        peers: 0,       // Updated by P2P layer
        mempool_size: 0, // Updated by transaction pool
        average_block_time: if height > 0 {
            let first_ts = blocks.first().map(|b| b.timestamp).unwrap_or(0);
            let last_ts = blocks.last().map(|b| b.timestamp).unwrap_or(0);
            if height > 1 { (last_ts - first_ts) as f64 / (height - 1) as f64 } else { 0.0 }
        } else { 0.0 },
        latest_blocks,
    };

    HttpResponse::Ok().json(stats)
}

/// Get block by hash or index
async fn get_block(
    path: web::Path<String>,
    data: web::Data<AppState>,
) -> impl Responder {
    let blocks = data.blocks.lock().unwrap();
    let identifier = path.into_inner();

    // Try parsing as index first
    if let Ok(index) = identifier.parse::<u64>() {
        if let Some(block) = blocks.iter().find(|b| b.index == index) {
            return HttpResponse::Ok().json(block);
        }
    }

    // Try as hash
    if let Some(block) = blocks.iter().find(|b| b.hash == identifier) {
        return HttpResponse::Ok().json(block);
    }

    HttpResponse::NotFound().json(serde_json::json!({
        "error": "Block not found"
    }))
}

/// Get latest blocks
async fn get_latest_blocks(
    query: web::Query<std::collections::HashMap<String, String>>,
    data: web::Data<AppState>,
) -> impl Responder {
    let blocks = data.blocks.lock().unwrap();
    let limit = query.get("limit")
        .and_then(|l| l.parse::<usize>().ok())
        .unwrap_or(20)
        .min(100);

    let latest: Vec<&Block> = blocks.iter().rev().take(limit).collect();
    HttpResponse::Ok().json(latest)
}

/// Get transaction by hash
async fn get_transaction(
    path: web::Path<String>,
    data: web::Data<AppState>,
) -> impl Responder {
    let transactions = data.transactions.lock().unwrap();
    let hash = path.into_inner();

    if let Some(tx) = transactions.iter().find(|t| t.hash == hash) {
        return HttpResponse::Ok().json(tx);
    }

    HttpResponse::NotFound().json(serde_json::json!({
        "error": "Transaction not found"
    }))
}

/// Get address information
async fn get_address(
    path: web::Path<String>,
    data: web::Data<AppState>,
) -> impl Responder {
    let transactions = data.transactions.lock().unwrap();
    let address = path.into_inner();

    // Find all transactions involving this address
    let addr_txs: Vec<Transaction> = transactions.iter()
        .filter(|tx| tx.sender == address || tx.recipient == address)
        .cloned()
        .collect();

    if addr_txs.is_empty() {
        return HttpResponse::NotFound().json(serde_json::json!({
            "error": "Address not found or has no transactions"
        }));
    }

    // Calculate balance and stats
    let mut balance: i64 = 0;
    let mut total_received: u64 = 0;
    let mut total_sent: u64 = 0;

    for tx in &addr_txs {
        if tx.recipient == address {
            balance += tx.amount as i64;
            total_received += tx.amount;
        }
        if tx.sender == address {
            balance -= (tx.amount + tx.fee) as i64;
            total_sent += tx.amount + tx.fee;
        }
    }

    let info = AddressInfo {
        address: address.clone(),
        balance: balance.max(0) as u64,
        total_received,
        total_sent,
        tx_count: addr_txs.len() as u32,
        recent_transactions: addr_txs.into_iter().take(20).collect(),
    };

    HttpResponse::Ok().json(info)
}

/// Search for block, transaction, or address
async fn search(
    path: web::Path<String>,
    data: web::Data<AppState>,
) -> impl Responder {
    let query = path.into_inner();
    let blocks = data.blocks.lock().unwrap();
    let transactions = data.transactions.lock().unwrap();

    // Try as block index
    if let Ok(index) = query.parse::<u64>() {
        if let Some(block) = blocks.iter().find(|b| b.index == index) {
            return HttpResponse::Ok().json(SearchResult::Block {
                data: block.clone(),
            });
        }
    }

    // Try as block hash
    if let Some(block) = blocks.iter().find(|b| b.hash == query) {
        return HttpResponse::Ok().json(SearchResult::Block {
            data: block.clone(),
        });
    }

    // Try as transaction hash
    if let Some(tx) = transactions.iter().find(|t| t.hash == query) {
        return HttpResponse::Ok().json(SearchResult::Transaction {
            data: tx.clone(),
        });
    }

    // Try as address
    let addr_txs: Vec<Transaction> = transactions.iter()
        .filter(|tx| tx.sender == query || tx.recipient == query)
        .cloned()
        .collect();

    if !addr_txs.is_empty() {
        let mut balance: i64 = 0;
        let mut total_received: u64 = 0;
        let mut total_sent: u64 = 0;

        for tx in &addr_txs {
            if tx.recipient == query {
                balance += tx.amount as i64;
                total_received += tx.amount;
            }
            if tx.sender == query {
                balance -= (tx.amount + tx.fee) as i64;
                total_sent += tx.amount + tx.fee;
            }
        }

        let info = AddressInfo {
            address: query.clone(),
            balance: balance.max(0) as u64,
            total_received,
            total_sent,
            tx_count: addr_txs.len() as u32,
            recent_transactions: addr_txs.into_iter().take(20).collect(),
        };

        return HttpResponse::Ok().json(SearchResult::Address { data: info });
    }

    HttpResponse::Ok().json(SearchResult::NotFound)
}

/// Health check endpoint
async fn health() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "service": "axiom-explorer-backend",
        "version": "1.0.0"
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    log::info!("Starting Axiom Explorer Backend...");

    let app_state = web::Data::new(AppState::new());

    HttpServer::new(move || {
        let cors = Cors::permissive(); // Allow all origins for development

        App::new()
            .wrap(cors)
            .app_data(app_state.clone())
            .route("/health", web::get().to(health))
            .route("/api/stats", web::get().to(get_stats))
            .route("/api/blocks", web::get().to(get_latest_blocks))
            .route("/api/block/{id}", web::get().to(get_block))
            .route("/api/transaction/{hash}", web::get().to(get_transaction))
            .route("/api/address/{address}", web::get().to(get_address))
            .route("/api/search/{query}", web::get().to(search))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_initialization() {
        let state = AppState::new();
        let blocks = state.blocks.lock().unwrap();
        assert!(!blocks.is_empty());
        assert_eq!(blocks[0].index, 0); // Genesis block
    }
}
