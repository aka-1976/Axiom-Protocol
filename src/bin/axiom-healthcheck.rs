use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

/// Health status response
#[derive(Serialize, Deserialize)]
struct HealthStatus {
    status: String,
    timestamp: u64,
    version: String,
    chain_height: u64,
    peers_connected: u32,
    mempool_size: u32,
    uptime_seconds: u64,
}

/// Metrics response
#[derive(Serialize, Deserialize)]
struct Metrics {
    // Chain metrics
    chain_height: u64,
    difficulty: u32,
    total_supply: u64,
    circulating_supply: u64,
    
    // Network metrics
    peers_connected: u32,
    peers_discovered: u32,
    inbound_connections: u32,
    outbound_connections: u32,
    
    // Transaction metrics
    mempool_size: u32,
    mempool_bytes: u64,
    transactions_processed: u64,
    transactions_per_second: f64,
    
    // Block metrics
    blocks_mined: u64,
    average_block_time: f64,
    last_block_time: u64,
    
    // System metrics
    uptime_seconds: u64,
    memory_usage_mb: f64,
    cpu_usage_percent: f64,
}

/// Prometheus-compatible metrics
#[allow(dead_code)]
#[derive(Serialize)]
struct PrometheusMetrics {
    metrics: Vec<PrometheusMetric>,
}

#[allow(dead_code)]
#[derive(Serialize)]
struct PrometheusMetric {
    name: String,
    value: f64,
    labels: std::collections::HashMap<String, String>,
    help: String,
    metric_type: String, // "gauge", "counter", "histogram"
}

/// Shared application state
struct AppState {
    start_time: SystemTime,
    chain_height: Arc<Mutex<u64>>,
    peers_connected: Arc<Mutex<u32>>,
    mempool_size: Arc<Mutex<u32>>,
    difficulty: Arc<Mutex<u32>>,
}

/// Health check endpoint
async fn health_check(data: web::Data<AppState>) -> impl Responder {
    let uptime = SystemTime::now()
        .duration_since(data.start_time)
        .unwrap()
        .as_secs();
    
    let chain_height = *data.chain_height.lock().unwrap();
    let peers = *data.peers_connected.lock().unwrap();
    let mempool = *data.mempool_size.lock().unwrap();
    
    // Determine health status
    let status = if peers > 0 && chain_height > 0 {
        "healthy"
    } else if chain_height > 0 {
        "degraded" // Chain working but no peers
    } else {
        "unhealthy"
    };
    
    let health = HealthStatus {
        status: status.to_string(),
        timestamp: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        chain_height,
        peers_connected: peers,
        mempool_size: mempool,
        uptime_seconds: uptime,
    };
    
    let status_code = match status {
        "healthy" => 200,
        "degraded" => 200, // Still return 200 but with degraded status
        _ => 503,
    };
    
    HttpResponse::build(actix_web::http::StatusCode::from_u16(status_code).unwrap())
        .json(health)
}

/// Metrics endpoint (JSON format)
async fn metrics(data: web::Data<AppState>) -> impl Responder {
    let uptime = SystemTime::now()
        .duration_since(data.start_time)
        .unwrap()
        .as_secs();
    
    let chain_height = *data.chain_height.lock().unwrap();
    let peers = *data.peers_connected.lock().unwrap();
    let mempool = *data.mempool_size.lock().unwrap();
    let diff = *data.difficulty.lock().unwrap();
    
    // Calculate some derived metrics
    let blocks_mined = chain_height.saturating_sub(1); // Exclude genesis
    let average_block_time = if blocks_mined > 0 {
        uptime as f64 / blocks_mined as f64
    } else {
        0.0
    };
    
    let metrics = Metrics {
        chain_height,
        difficulty: diff,
        total_supply: 124_000_000_000_000_000, // 124M AXM in smallest units
        circulating_supply: axiom_core::economics::cumulative_supply_at_block(blocks_mined),
        
        peers_connected: peers,
        peers_discovered: peers, // Tracked peers = connected peers at discovery layer
        inbound_connections: 0,  // Tracked separately by libp2p swarm metrics
        outbound_connections: peers,
        
        mempool_size: mempool,
        mempool_bytes: (mempool as u64) * 500, // ~500 bytes avg per serialized tx
        transactions_processed: 0, // Requires chain scan — omitted for low-latency metrics
        transactions_per_second: 0.0,
        
        blocks_mined,
        average_block_time,
        last_block_time: uptime, // Would need to track separately
        
        uptime_seconds: uptime,
        memory_usage_mb: 0.0, // Would need sys-info crate
        cpu_usage_percent: 0.0, // Would need sys-info crate
    };
    
    HttpResponse::Ok().json(metrics)
}

/// Prometheus-compatible metrics endpoint
async fn prometheus_metrics(data: web::Data<AppState>) -> impl Responder {
    let uptime = SystemTime::now()
        .duration_since(data.start_time)
        .unwrap()
        .as_secs();
    
    let chain_height = *data.chain_height.lock().unwrap();
    let peers = *data.peers_connected.lock().unwrap();
    let mempool = *data.mempool_size.lock().unwrap();
    let diff = *data.difficulty.lock().unwrap();
    
    // Generate Prometheus format
    let mut output = String::new();
    
    // Chain metrics
    output.push_str("# HELP axiom_chain_height Current blockchain height\n");
    output.push_str("# TYPE axiom_chain_height gauge\n");
    output.push_str(&format!("axiom_chain_height {}\n", chain_height));
    
    output.push_str("# HELP axiom_difficulty Current mining difficulty\n");
    output.push_str("# TYPE axiom_difficulty gauge\n");
    output.push_str(&format!("axiom_difficulty {}\n", diff));
    
    // Network metrics
    output.push_str("# HELP axiom_peers_connected Number of connected peers\n");
    output.push_str("# TYPE axiom_peers_connected gauge\n");
    output.push_str(&format!("axiom_peers_connected {}\n", peers));
    
    // Transaction metrics
    output.push_str("# HELP axiom_mempool_size Number of transactions in mempool\n");
    output.push_str("# TYPE axiom_mempool_size gauge\n");
    output.push_str(&format!("axiom_mempool_size {}\n", mempool));
    
    // Block metrics
    let blocks_mined = chain_height.saturating_sub(1);
    output.push_str("# HELP axiom_blocks_mined_total Total blocks mined\n");
    output.push_str("# TYPE axiom_blocks_mined_total counter\n");
    output.push_str(&format!("axiom_blocks_mined_total {}\n", blocks_mined));
    
    // System metrics
    output.push_str("# HELP axiom_uptime_seconds Node uptime in seconds\n");
    output.push_str("# TYPE axiom_uptime_seconds counter\n");
    output.push_str(&format!("axiom_uptime_seconds {}\n", uptime));
    
    // Supply metrics
    let circulating = blocks_mined * 5_000_000_000;
    output.push_str("# HELP axiom_circulating_supply Circulating supply in smallest units\n");
    output.push_str("# TYPE axiom_circulating_supply gauge\n");
    output.push_str(&format!("axiom_circulating_supply {}\n", circulating));
    
    HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4")
        .body(output)
}

/// Readiness probe (for Kubernetes)
async fn readiness(data: web::Data<AppState>) -> impl Responder {
    // Check chain sync status and peer connectivity
    let chain_height = *data.chain_height.lock().unwrap();
    let peers = *data.peers_connected.lock().unwrap();
    let ready = chain_height > 0 && peers > 0;

    if ready {
        HttpResponse::Ok().json(serde_json::json!({
            "ready": true,
            "message": "Node is synced and has peer connections",
            "chain_height": chain_height,
            "peers": peers
        }))
    } else {
        HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "ready": false,
            "message": "Node is still syncing or has no peers",
            "chain_height": chain_height,
            "peers": peers
        }))
    }
}

/// Liveness probe (for Kubernetes)
async fn liveness() -> impl Responder {
    // Simple check that the process is alive
    HttpResponse::Ok().json(serde_json::json!({
        "alive": true,
        "timestamp": SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }))
}

/// Info endpoint with detailed node information
async fn info(data: web::Data<AppState>) -> impl Responder {
    let chain_height = *data.chain_height.lock().unwrap();
    let peers = *data.peers_connected.lock().unwrap();
    
    HttpResponse::Ok().json(serde_json::json!({
        "name": "AXIOM Protocol Node",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "AXIOM Protocol - 124M Sovereign Scarcity Engine",
        "chain_height": chain_height,
        "peers": peers,
        "network": "mainnet",
        "consensus": "VDF+PoW Hybrid",
        "privacy": "ZK-STARK",
        "supply": {
            "total": 84_000_000,
            "unit": "AXM"
        }
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("🏥 Starting Axiom Health & Metrics Server...");
    
    let state = web::Data::new(AppState {
        start_time: SystemTime::now(),
        chain_height: Arc::new(Mutex::new(0)),
        peers_connected: Arc::new(Mutex::new(0)),
        mempool_size: Arc::new(Mutex::new(0)),
        difficulty: Arc::new(Mutex::new(1000)),
    });
    
    let bind_address = "0.0.0.0:9090";
    println!("🌐 Listening on http://{}", bind_address);
    println!("📊 Endpoints:");
    println!("   GET /health         - Health check (JSON)");
    println!("   GET /metrics        - Metrics (JSON)");
    println!("   GET /metrics/prometheus - Prometheus format");
    println!("   GET /readiness      - Readiness probe");
    println!("   GET /liveness       - Liveness probe");
    println!("   GET /info           - Node information");
    
    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .route("/health", web::get().to(health_check))
            .route("/metrics", web::get().to(metrics))
            .route("/metrics/prometheus", web::get().to(prometheus_metrics))
            .route("/readiness", web::get().to(readiness))
            .route("/liveness", web::get().to(liveness))
            .route("/info", web::get().to(info))
    })
    .bind(bind_address)?
    .run()
    .await
}
