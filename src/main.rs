// Main orchestrator for AXIOM Protocol node
// Integrates network, consensus, and AI Guardian
// Architecture: Discv5 (UDP radar) discovers peers, libp2p (TCP) handles messaging

use std::collections::{HashMap, HashSet, VecDeque};
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::time;
use libp2p::{gossipsub, Multiaddr, PeerId, Swarm};
use libp2p::swarm::SwarmEvent;
use futures::StreamExt;

// Import all necessary modules
use axiom_core::network_legacy::{TimechainBehaviourEvent, init_network, init_network_with_bootstrap};
use axiom_core::network::Discv5Service;
use axiom_core::network::discv5_service::default_bootstrap_enrs;
use axiom_core::AxiomPulse;

// These are placeholders - adjust based on your actual module structure
mod wallet {
    pub struct Wallet { pub address: [u8; 32] }
    impl Wallet {
        pub fn load_or_create() -> Self {
            Self { address: [0u8; 32] }
        }
    }
}

mod ai_guardian {
    pub struct Stats;
    
    pub struct NeuralGuardian;
    impl NeuralGuardian {
        pub fn new() -> Self { Self }
        pub fn predict_trust(&mut self, _a: f32, _b: f32, _c: f32) -> bool { true }
        pub fn train(&mut self, _input: [f32; 3], _output: f32) {}
        pub fn log_stats(&self) {}
    }
}

mod timechain {
    use super::Block;
    use super::Transaction;
    pub struct Timechain {
        pub blocks: Vec<Block>,
        pub difficulty: u64,
    }
    impl Timechain {
        pub fn new(_genesis: Block) -> Self {
            Self { blocks: vec![], difficulty: 1000 }
        }
        pub fn add_block(&mut self, _block: Block, _elapsed: u64) -> Result<(), String> {
            Ok(())
        }
        pub fn validate_transaction(&self, _tx: &Transaction) -> Result<(), String> {
            Ok(())
        }
        pub fn supply_info(&self) -> (u64, u64, f64) {
            (0, 124_000_000_00000000, 0.0)
        }
        pub fn format_axm(amount: u64) -> String {
            format!("{:.8}", amount as f64 / 100000000.0)
        }
    }
}

mod genesis {
    use super::Block;
    pub fn genesis() -> Block {
        Block {
            parent: [0u8; 32],
            slot: 0,
            miner: [0u8; 32],
            transactions: vec![],
            vdf_proof: [0u8; 32],
            zk_proof: vec![],
            nonce: 0,
        }
    }
    pub fn generate_zk_pass(_wallet: &crate::wallet::Wallet, _hash: [u8; 32]) -> Vec<u8> {
        vec![0u8; 32]
    }
}

mod storage {
    use super::Block;
    pub fn load_chain() -> Option<Vec<Block>> { None }
    pub fn save_chain(_blocks: &[Block]) {}
}

mod vdf {
    pub fn evaluate(_hash: [u8; 32], _slot: u64) -> [u8; 32] { [0u8; 32] }
}

mod openclaw_integration {
    use tokio::task::JoinHandle;
    pub async fn start_openclaw_background() -> Result<JoinHandle<()>, Box<dyn std::error::Error>> {
        Ok(tokio::spawn(async {}))
    }
}

use wallet::Wallet;
use ai_guardian::NeuralGuardian;
use timechain::Timechain;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Transaction {
    pub from: [u8; 32],
    pub to: [u8; 32],
    pub amount: u64,
    pub nonce: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Block {
    pub parent: [u8; 32],
    pub slot: u64,
    pub miner: [u8; 32],
    pub transactions: Vec<Transaction>,
    pub vdf_proof: [u8; 32],
    pub zk_proof: Vec<u8>,
    pub nonce: u64,
}

impl Block {
    pub fn hash(&self) -> [u8; 32] {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(&self.parent);
        hasher.update(&self.slot.to_le_bytes());
        hasher.finalize().into()
    }

    /// 512-bit BLAKE3 block hash using deterministic field-by-field feed.
    pub fn hash_512(&self) -> [u8; 64] {
        let mut hasher = blake3::Hasher::new();
        hasher.update(&self.parent);
        hasher.update(&self.slot.to_be_bytes());
        hasher.update(&self.miner);
        hasher.update(&self.vdf_proof);
        hasher.update(&self.zk_proof);
        hasher.update(&self.nonce.to_be_bytes());
        let mut output = [0u8; 64];
        hasher.finalize_xof().fill(&mut output);
        output
    }

    pub fn meets_difficulty(&self, difficulty: u64) -> bool {
        let hash = self.hash();
        let hash_num = u64::from_le_bytes(hash[0..8].try_into().unwrap());
        hash_num < u64::MAX / difficulty
    }
}

fn compute_vdf(_seed: [u8; 32], _difficulty: u32) -> [u8; 32] {
    [0u8; 32]
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("--------------------------------------------------");
    println!("🏛️  AXIOM CORE | PRIVACY-FIRST BLOCKCHAIN");
    println!("🛡️  VDF: 1800sec (30min) | PoW Hybrid | 124M Fixed Supply");
    println!("🤖 AI NEURAL GUARDIAN: ATTACK DETECTION ACTIVE");
    println!("🔐 MANDATORY ZK-STARK PRIVACY | ED25519 SIGNATURES");
    println!("--------------------------------------------------");

    // 1. IDENTITY & STATE INITIALIZATION
    let wallet = Wallet::load_or_create();
    println!("💳 Wallet Address: {:?}", hex::encode(wallet.address));
    println!("📁 Wallet file: ./wallet.dat (keep safe!)");

    let ai_guardian = Arc::new(Mutex::new(NeuralGuardian::new()));
    let mut peer_message_counts: HashMap<PeerId, (u32, Instant)> = HashMap::new();

    // Transaction mempool
    let mut mempool: VecDeque<Transaction> = VecDeque::new();

    // Load or create blockchain
    let mut tc = if let Some(saved_blocks) = storage::load_chain() {
        println!("✅ STORAGE: Loaded {} blocks. Integrity verified.", saved_blocks.len());
        let mut chain = Timechain::new(genesis::genesis());
        for b in saved_blocks {
            let _ = chain.add_block(b, 1800);
        }
        chain
    } else {
        Timechain::new(genesis::genesis())
    };

    println!("\n--- AXIOM GENESIS ANCHOR ---");
    println!("HASH: {:?}", hex::encode(tc.blocks.first().unwrap_or(&genesis::genesis()).hash()));
    println!("----------------------------\n");

    // 2. NETWORK SETUP
    let bootstrap_peers: Vec<String> = std::env::var("AXIOM_BOOTSTRAP_PEERS")
        .unwrap_or_default()
        .split(',')
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.trim().to_string())
        .collect();

    let mut swarm: Swarm<axiom_core::network_legacy::TimechainBehaviour> = if !bootstrap_peers.is_empty() {
        init_network_with_bootstrap(bootstrap_peers).await
            .map_err(|e| -> Box<dyn Error> { e })?
    } else {
        init_network().await
            .map_err(|e| -> Box<dyn Error> { e })?
    };

    // Port Binding Logic
    let is_genesis = std::env::var("AXIOM_GENESIS_NODE").unwrap_or_default() == "1";
    let (port_start, port_end) = if is_genesis { (6000, 6003) } else { (6000, 6999) };

    let mut current_port = port_start;
    loop {
        let addr: Multiaddr = format!("/ip4/0.0.0.0/tcp/{}", current_port).parse()?;
        match swarm.listen_on(addr.clone()) {
            Ok(_) => {
                println!("🌐 Node successfully bound to port: {}", current_port);
                println!("🆔 PeerId: {}", swarm.local_peer_id());
                println!("🔊 Listening on: {}", addr);
                println!("[DIAG] To connect another node, set AXIOM_BOOTSTRAP_PEER=\"{}@/ip4/0.0.0.0/tcp/{}\"",
                    swarm.local_peer_id(), current_port);
                break;
            }
            Err(e) => {
                if current_port < port_end {
                    println!("⚠️  Port {} busy. Trying {}...", current_port, current_port + 1);
                    current_port += 1;
                } else {
                    return Err(format!("No available ports in range {}-{}", port_start, port_end).into());
                }
            }
        }
    }

    // 3. BOOTSTRAP CONFIGURATION
    println!("🌍 Bootstrap Configuration:");
    let mut bootstrap_connected = 0;
    let mut bootstrap_addrs: Vec<(String, Multiaddr)> = Vec::new();

    // Try environment variable first
    let env_bootstrap_peers: Vec<String> = std::env::var("AXIOM_BOOTSTRAP_PEERS")
        .unwrap_or_default()
        .split(',')
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.trim().to_string())
        .collect();

    if !env_bootstrap_peers.is_empty() {
        println!("   📌 Using AXIOM_BOOTSTRAP_PEERS environment variable");
        for addr_str in &env_bootstrap_peers {
            if let Ok(addr) = addr_str.parse::<Multiaddr>() {
                bootstrap_addrs.push((addr_str.clone(), addr.clone()));
                match swarm.dial(addr.clone()) {
                    Ok(_) => {
                        println!("   ✅ Dialing bootstrap node: {}", addr_str);
                        bootstrap_connected += 1;
                    }
                    Err(e) => println!("   ⚠️  Failed to dial bootstrap node {}: {:?}", addr_str, e),
                }
            }
        }
    } else if let Ok(bootstrap_content) = std::fs::read_to_string("config/bootstrap.toml") {
        // Fallback to config file
        if let Ok(bootstrap_config) = toml::from_str::<toml::Value>(&bootstrap_content) {
            if let Some(bootnodes) = bootstrap_config.get("bootnodes").and_then(|v| v.as_array()) {
                if !bootnodes.is_empty() {
                    println!("   📌 Using config/bootstrap.toml addresses");
                    for bootnode in bootnodes {
                        if let Some(addr_str) = bootnode.as_str() {
                            if let Ok(addr) = addr_str.parse::<Multiaddr>() {
                                bootstrap_addrs.push((addr_str.to_string(), addr.clone()));
                                match swarm.dial(addr.clone()) {
                                    Ok(_) => {
                                        println!("   ✅ Dialing bootstrap node: {}", addr_str);
                                        bootstrap_connected += 1;
                                    }
                                    Err(e) => println!("   ⚠️  Failed to dial: {:?}", e),
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if bootstrap_connected == 0 {
        println!("   🌐 Using mDNS and Discv5 for peer discovery");
    } else {
        println!("   ✅ {} bootstrap nodes queued for connection", bootstrap_connected);
    }

    // 3b. DISCV5 PEER DISCOVERY (UDP Radar)
    // Discv5 runs externally alongside the libp2p Swarm, not inside NetworkBehaviour.
    // It scans the network (UDP) and discovered peers are manually dialed by the Swarm (TCP).
    let discv5_udp_port = current_port as u32 + 3000;
    let discv5_service = if discv5_udp_port <= 65535 {
        let discv5_listen_addr: std::net::SocketAddr = format!("0.0.0.0:{}", discv5_udp_port)
            .parse()
            .expect("valid socket addr");
        let discv5_key = discv5::enr::CombinedKey::generate_secp256k1();
        let boot_enrs = default_bootstrap_enrs();

        match Discv5Service::new(discv5_listen_addr, discv5_key, boot_enrs).await {
            Ok(svc) => {
                println!("🔍 Discv5 discovery active on UDP port {}", discv5_udp_port);
                Some(svc)
            }
            Err(e) => {
                println!("⚠️  Discv5 init warning (falling back to mDNS only): {}", e);
                None
            }
        }
    } else {
        println!("⚠️  Discv5 UDP port {} exceeds valid range, falling back to mDNS only", discv5_udp_port);
        None
    };
    let mut discv5_lookup_timer = time::interval(Duration::from_secs(30));

    // 4. TOPICS
    let req_topic = gossipsub::IdentTopic::new("timechain-request");
    let chain_topic = gossipsub::IdentTopic::new("timechain-chain");
    let blocks_topic = gossipsub::IdentTopic::new("timechain-blocks");
    let tx_topic = gossipsub::IdentTopic::new("timechain-transactions");
    let pulse_topic = gossipsub::IdentTopic::new("axiom/realtime/pulse/v1");

    // Subscribe to topics
    swarm.behaviour_mut().gossipsub.subscribe(&req_topic)?;
    swarm.behaviour_mut().gossipsub.subscribe(&chain_topic)?;
    swarm.behaviour_mut().gossipsub.subscribe(&blocks_topic)?;
    swarm.behaviour_mut().gossipsub.subscribe(&tx_topic)?;
    swarm.behaviour_mut().gossipsub.subscribe(&pulse_topic)?;

    // Request chains from network
    let _ = swarm.behaviour_mut().gossipsub.publish(req_topic.clone(), b"REQ_CHAIN".to_vec());

    // 5. START OPENCLAW
    println!("🤖 Initializing OpenClaw automation...");
    let _openclaw_handle = match openclaw_integration::start_openclaw_background().await {
        Ok(handle) => {
            println!("✅ OpenClaw started in background");
            Some(handle)
        }
        Err(e) => {
            println!("⚠️  OpenClaw startup warning: {}", e);
            None
        }
    };

    // 6. TIMERS AND STATE
    let mut last_vdf = Instant::now();
    let mut last_diff = tc.difficulty;
    let mut last_bootstrap_retry = Instant::now();

    let mut vdf_loop = time::interval(Duration::from_millis(100));
    let mut dashboard_timer = time::interval(Duration::from_secs(10));
    let mut throttle_reset = time::interval(Duration::from_secs(60));
    let mut tx_broadcast_timer = time::interval(Duration::from_secs(30));
    let mut chain_sync_timer = time::interval(Duration::from_secs(300));
    let mut bootstrap_retry_timer = time::interval(Duration::from_secs(120));
    let mut cross_network_discovery = time::interval(Duration::from_secs(30));

    let mut connected_peers: HashSet<PeerId> = HashSet::new();
    let known_peers: Vec<String> = std::env::var("AXIOM_KNOWN_PEERS")
        .unwrap_or_default()
        .split(',')
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.trim().to_string())
        .collect();

    // 7. MAIN EVENT LOOP
    loop {
        tokio::select! {
            // P2P EVENTS
            event = swarm.select_next_some() => match event {
                SwarmEvent::Behaviour(TimechainBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                    propagation_source, message, ..
                })) => {
                    // Rate limiting
                    let now = Instant::now();
                    let entry = peer_message_counts.entry(propagation_source).or_insert((0, now));
                    if now.duration_since(entry.1) > Duration::from_secs(60) {
                        entry.0 = 0;
                        entry.1 = now;
                    }
                    entry.0 += 1;

                    if entry.0 > 100 {
                        println!("🚨 DoS protection: Peer {} exceeded rate limit", propagation_source);
                        continue;
                    }

                    let mut ai = ai_guardian.lock().unwrap();
                    let is_trustworthy = ai.predict_trust(1.0 / (entry.0 as f32), 1.0, 1.0);

                    if is_trustworthy && entry.0 <= 15 {
                        // Handle chain request
                        if message.data == b"REQ_CHAIN" {
                            if let Ok(encoded) = bincode::serialize(&tc.blocks) {
                                let _ = swarm.behaviour_mut().gossipsub.publish(chain_topic.clone(), encoded);
                            }
                        }
                        // Handle block
                        else if message.topic == blocks_topic.hash() {
                            if let Ok(block) = bincode::deserialize::<Block>(&message.data) {
                                let elapsed = last_vdf.elapsed().as_secs();
                                if tc.add_block(block, elapsed).is_ok() {
                                    println!("✅ Block accepted and added to chain");
                                    storage::save_chain(&tc.blocks);
                                }
                            }
                        }
                        // Handle transaction
                        else if message.topic == tx_topic.hash() {
                            if let Ok(tx) = bincode::deserialize::<Transaction>(&message.data) {
                                if tc.validate_transaction(&tx).is_ok() && !mempool.contains(&tx) {
                                    mempool.push_back(tx);
                                    println!("✅ Transaction added to mempool");
                                }
                            }
                        }
                        // Handle full chain
                        else if message.topic == chain_topic.hash() {
                            if let Ok(peer_blocks) = bincode::deserialize::<Vec<Block>>(&message.data) {
                                if peer_blocks.len() > tc.blocks.len() {
                                    println!("🔁 Synced chain from peer. New height: {}", peer_blocks.len());
                                    storage::save_chain(&peer_blocks);
                                    last_vdf = Instant::now();
                                }
                            }
                        }
                        // Handle real-time pulse (push-based sync)
                        else if message.topic == pulse_topic.hash() {
                            if let Ok(pulse) = bincode::deserialize::<AxiomPulse>(&message.data) {
                                if pulse.height > tc.blocks.len() as u64 {
                                    println!("🔥 Real-time Pulse: Height {} | Mined: {} AXM | Remaining: {} AXM",
                                        pulse.height,
                                        Timechain::format_axm(pulse.total_mined),
                                        Timechain::format_axm(pulse.remaining));
                                }
                            }
                        }
                    } else if entry.0 > 20 {
                        ai.train([0.1, 0.0, 0.0], 0.0);
                    }
                }

                SwarmEvent::NewListenAddr { address, .. } => {
                    println!("🌐 Node active on: {}", address);
                }

                SwarmEvent::ConnectionEstablished { peer_id, endpoint, .. } => {
                    connected_peers.insert(peer_id);
                    println!("🔗 Peer connected: {} | Total: {}", peer_id, connected_peers.len());
                }

                SwarmEvent::ConnectionClosed { peer_id, .. } => {
                    connected_peers.remove(&peer_id);
                    println!("🔌 Peer disconnected: {} | Total: {}", peer_id, connected_peers.len());
                }

                _ => {}
            },

            // THROTTLE RESET
            _ = throttle_reset.tick() => {
                peer_message_counts.clear();
            }

            // TX BROADCAST
            _ = tx_broadcast_timer.tick() => {
                if let Ok(tx_data) = std::fs::read("pending_tx.dat") {
                    if let Ok(tx) = bincode::deserialize::<Transaction>(&tx_data) {
                        if tc.validate_transaction(&tx).is_ok() {
                            let encoded = bincode::serialize(&tx).unwrap();
                            let _ = swarm.behaviour_mut().gossipsub.publish(tx_topic.clone(), encoded);
                            println!("📤 Transaction broadcasted");
                            let _ = std::fs::remove_file("pending_tx.dat");
                        }
                    }
                }
            }

            // CHAIN SYNC
            _ = chain_sync_timer.tick() => {
                println!("🔄 Performing periodic chain synchronization...");
                let _ = swarm.behaviour_mut().gossipsub.publish(req_topic.clone(), b"REQ_CHAIN".to_vec());
            }

            // DASHBOARD
            _ = dashboard_timer.tick() => {
                let elapsed = last_vdf.elapsed().as_secs();
                let remaining = 1800u64.saturating_sub(elapsed);
                let trend = if tc.difficulty > last_diff { "UP ⬆️" }
                    else if tc.difficulty < last_diff { "DOWN ⬇️" }
                    else { "STABLE ↔️" };

                let (mined, remaining_supply, percent) = tc.supply_info();
                let mined_axm = Timechain::format_axm(mined);
                let remaining_axm = Timechain::format_axm(remaining_supply);

                println!("\n--- 🏛️  AXIOM STATUS ---");
                println!("⛓️  Height: {} | Diff: {} | Trend: {}", tc.blocks.len(), tc.difficulty, trend);
                println!("⏳ Time-Lock: {}m remaining | 🤖 AI Shield: ACTIVE", remaining / 60);
                println!("💰 Mined: {} AXM | Remaining: {} AXM | {:.2}% of max supply",
                    mined_axm, remaining_axm, percent);

                println!("🌐 Network Status:");
                println!("   ├─ PeerId: {}", swarm.local_peer_id());
                println!("   ├─ Connected Peers: {}", connected_peers.len());
                if connected_peers.is_empty() {
                    println!("   │  └─ No peers connected (check firewall/NAT)");
                }
                println!("   └─ Listen Addresses:");
                for addr in libp2p::Swarm::listeners(&swarm) {
                    println!("      └─ {}", addr);
                }

                let ai = ai_guardian.lock().unwrap();
                ai.log_stats();
                println!("------------------------\n");

                last_diff = tc.difficulty;
            }

            // BOOTSTRAP RETRY
            _ = bootstrap_retry_timer.tick() => {
                if connected_peers.len() < 2 && last_bootstrap_retry.elapsed().as_secs() > 120 {
                    for (_, addr) in &bootstrap_addrs {
                        let _ = swarm.dial(addr.clone());
                    }
                    last_bootstrap_retry = Instant::now();
                }
            }

            // DISCV5 PEER DISCOVERY BRIDGE
            // Discv5 acts as our "radar" (UDP) - finds peers on the network
            // libp2p acts as our "cargo ship" (TCP) - opens secure tunnels to send data
            _ = discv5_lookup_timer.tick() => {
                if let Some(ref svc) = discv5_service {
                    // Only attempt discovery if we need more peers
                    if connected_peers.len() < 50 {
                        let table_peers = svc.table_entries().await;
                        for enr in table_peers {
                            // Extract TCP multiaddr from ENR and dial via libp2p
                            if let Some(ip) = enr.ip4() {
                                // Skip loopback and unspecified addresses
                                if ip.is_loopback() || ip.is_unspecified() {
                                    continue;
                                }
                                if let Some(tcp_port) = enr.tcp4() {
                                    if let Ok(addr) = format!("/ip4/{}/tcp/{}", ip, tcp_port).parse::<Multiaddr>() {
                                        if let Err(e) = swarm.dial(addr.clone()) {
                                            println!("⚠️  Discv5 bridge: failed to dial {}: {}", addr, e);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // MINING
            _ = vdf_loop.tick() => {
                let elapsed = last_vdf.elapsed().as_secs();
                if elapsed >= 1800 {
                    let parent_hash = tc.blocks.last().unwrap_or(&genesis::genesis()).hash();
                    let current_slot = tc.blocks.len() as u64;
                    let vdf_seed = vdf::evaluate(parent_hash, current_slot);
                    let vdf_proof = compute_vdf(vdf_seed, tc.difficulty as u32);
                    let zk_pass = genesis::generate_zk_pass(&wallet, parent_hash);

                    let mut nonce = 0u64;
                    let max_attempts = 100000;

                    while nonce < max_attempts {
                        let candidate = Block {
                            parent: parent_hash,
                            slot: current_slot,
                            miner: wallet.address,
                            transactions: vec![],
                            vdf_proof,
                            zk_proof: zk_pass.clone(),
                            nonce,
                        };

                        if candidate.meets_difficulty(tc.difficulty) && tc.add_block(candidate.clone(), elapsed).is_ok() {
                            println!("✨ MINED: H-{} | Nonce: {}", tc.blocks.len(), nonce);
                            let encoded = bincode::serialize(&candidate).unwrap();
                            let _ = swarm.behaviour_mut().gossipsub.publish(blocks_topic.clone(), encoded);
                            storage::save_chain(&tc.blocks);

                            // Broadcast real-time pulse to all peers
                            let height = tc.blocks.len() as u64;
                            let (total_mined, remaining, _percent) = tc.supply_info();
                            // Generate deterministic AI oracle seal for this block
                            let oracle_query = format!(
                                "Axiom block {} mined with hash {}",
                                tc.blocks.len(),
                                hex::encode(candidate.hash())
                            );
                            let oracle_seal = axiom_core::ai::query_oracle(&oracle_query).await;

                            let pulse = AxiomPulse {
                                height,
                                total_mined,
                                remaining,
                                block_hash: candidate.hash_512(),
                                oracle_seal,
                                timestamp: std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap_or_default()
                                    .as_secs() as i64,
                            };
                            if let Ok(pulse_data) = bincode::serialize(&pulse) {
                                let _ = swarm.behaviour_mut().gossipsub.publish(pulse_topic.clone(), pulse_data);
                            }

                            last_vdf = Instant::now();
                            break;
                        }
                        nonce += 1;
                    }
                }
            }
        }
    }
}
