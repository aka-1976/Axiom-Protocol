#![allow(unused_imports)]
#![allow(dead_code)]


use axiom_core::{block, transaction, chain, network, storage, main_helper, genesis, bridge, vdf, ai_engine, state, economics, wallet, zk, openclaw_integration};
use axiom_core::zk::circuit;

use block::Block;
use chain::Timechain;
use transaction::Transaction;
use ai_engine::NeuralGuardian;
use serde_json;
use wallet::Wallet;
use main_helper::compute_vdf;
use libp2p::{gossipsub, swarm::SwarmEvent, futures::StreamExt, Multiaddr, PeerId};
use std::time::{Duration, Instant};
use tokio::time;
use std::error::Error;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex};

/// Enhanced chain validation and synchronization for global consensus
fn validate_and_sync_chain(peer_blocks: &[Block], current_chain: &Timechain) -> Option<Timechain> {
    if peer_blocks.is_empty() {
        return None;
    }

    // Verify genesis block matches
    if peer_blocks[0].hash() != current_chain.blocks[0].hash() {
        println!("⚠️  Peer chain has different genesis block - rejecting");
        return None;
    }

    // Try to reconstruct and validate the peer's chain
    let mut candidate = Timechain::new(genesis::genesis());
    let mut valid = true;

    for (i, block) in peer_blocks.iter().enumerate().skip(1) {
        // Validate block structure and consensus rules
        if candidate.add_block(block.clone(), 1800).is_err() {
            println!("⚠️  Invalid block at height {} from peer - rejecting chain", i);
            valid = false;
            break;
        }
    }

    if !valid {
        return None;
    }

    // Accept the chain if it's longer or has more work (for tie-breaking)
    let peer_work = calculate_chain_work(&candidate);
    let current_work = calculate_chain_work(current_chain);

    if candidate.blocks.len() > current_chain.blocks.len() || peer_work > current_work {
        println!("✅ Peer chain validated - Work: {} vs {}", peer_work, current_work);
        Some(candidate)
    } else {
        None
    }
}

/// Calculate total work (cumulative difficulty) of a chain
fn calculate_chain_work(chain: &Timechain) -> u64 {
    chain.blocks.iter().map(|block| block.nonce.max(1)).sum()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("--------------------------------------------------");
    println!("🏛️  AXIOM CORE | PRIVACY-FIRST BLOCKCHAIN");
    println!("🛡️  VDF: 1800sec (30min) | PoW Hybrid | 124M Fixed Supply");
    println!("🤖 AI NEURAL GUARDIAN: ATTACK DETECTION ACTIVE");
    println!("🔐 MANDATORY ZK-SNARK PRIVACY | ED25519 SIGNATURES");
    println!("--------------------------------------------------");
    println!("📅 February 2026 | OpenClaw Integration Active");

    // --- Network Diagnostics: Check bootstrap connectivity on startup ---
    network::check_bootstrap_connectivity();

    // 1. IDENTITY & STATE INITIALIZATION
    let wallet = Wallet::load_or_create();
    println!("💳 Wallet Address: {:?}", hex::encode(wallet.address));
    println!("📁 Wallet file: ./wallet.dat (keep safe!)");
    let ai_guardian = Arc::new(Mutex::new(NeuralGuardian::new()));
    let mut peer_message_counts: HashMap<PeerId, (u32, Instant)> = HashMap::new();

    // Transaction mempool
    let mut mempool: VecDeque<Transaction> = VecDeque::new();

    let mut tc = if let Some(saved_blocks) = storage::load_chain() {
        let mut chain = Timechain::new(genesis::genesis());
        for b in saved_blocks { let _ = chain.add_block(b, 1800); }
        chain
    } else {
        Timechain::new(genesis::genesis())
    };

    // 2. NETWORK SETUP
    // --- Network Setup with Dynamic Port Hunting and Bootstrap Peers ---
    let bootstrap_peers: Vec<String> = std::env::var("AXIOM_BOOTSTRAP_PEERS")
        .unwrap_or_default()
        .split(',')
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.trim().to_string())
        .collect();
    let mut swarm = if !bootstrap_peers.is_empty() {
        network::init_network_with_bootstrap(bootstrap_peers).await?
    } else {
        network::init_network().await?
    };

    let mut current_port: u16 = 6000;
    let max_port: u16 = 6003; // Restricted to 4 genesis mining nodes only

    loop {
        let addr: Multiaddr = format!("/ip4/0.0.0.0/tcp/{}", current_port).parse()?;
        match swarm.listen_on(addr.clone()) {
            Ok(_) => {
                println!("🌐 Node successfully bound to port: {}", current_port);
                // --- BEGIN DIAGNOSTICS ---
                println!("🆔 PeerId: {}", swarm.local_peer_id());
                println!("🔊 Listening on: {}", addr);
                for laddr in libp2p::Swarm::listeners(&swarm) {
                    println!("🔊 Swarm listening: {}", laddr);
                }
                println!("[DIAG] To connect another node, set AXIOM_BOOTSTRAP_PEER=\"{}@{}\"", swarm.local_peer_id(), addr);
                // --- END DIAGNOSTICS ---
                break;
            }
            Err(e) => {
                if current_port < max_port {
                    println!("⚠️  Port {} busy. Trying {}...", current_port, current_port + 1);
                    current_port += 1;
                } else {
                    println!("❌ Critical Error: No available ports found in range 6000-6003 (4 genesis nodes max).");
                    return Err(e.into());
                }
            }
        }
    }

    // Subscribe to gossip topics used for block propagation and chain sync
    let blocks_topic = gossipsub::IdentTopic::new("timechain-blocks");
    let req_topic = gossipsub::IdentTopic::new("timechain-requests");
    let chain_topic = gossipsub::IdentTopic::new("timechain-chain");
    let tx_topic = gossipsub::IdentTopic::new("timechain-transactions");
    let _ = swarm.behaviour_mut().gossipsub.subscribe(&blocks_topic);
    let _ = swarm.behaviour_mut().gossipsub.subscribe(&req_topic);
    let _ = swarm.behaviour_mut().gossipsub.subscribe(&chain_topic);
    let _ = swarm.behaviour_mut().gossipsub.subscribe(&tx_topic);

    // 3. BOOTSTRAP CONNECTIONS - Connect to mainnet bootnodes for global sync
    println!("🌍 Bootstrap Configuration:");
    let mut bootstrap_addrs = Vec::new();
    let mut bootstrap_connected = 0;
    
    // First, try environment variable for dynamic bootstrap peers
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
                        if let Some(peer_str) = addr_str.split("/p2p/").nth(1) {
                            if let Ok(peer_id) = peer_str.parse::<libp2p::PeerId>() {
                                swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);
                            }
                        }
                    },
                    Err(e) => println!("   ⚠️  Failed to dial bootstrap node {}: {:?}", addr_str, e),
                }
            }
        }
    } else if let Ok(bootstrap_content) = std::fs::read_to_string("config/bootstrap.toml") {
        // Fall back to config file
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
                                        if let Some(peer_str) = addr_str.split("/p2p/").nth(1) {
                                            if let Ok(peer_id) = peer_str.parse::<libp2p::PeerId>() {
                                                swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);
                                            }
                                        }
                                    },
                                    Err(e) => println!("   ⚠️  Failed to dial bootstrap node {}: {:?}", addr_str, e),
                                }
                            }
                        }
                    }
                } else {
                    println!("   📌 No bootstrap nodes configured (config/bootstrap.toml is empty)");
                }
            }
        }
    } else {
        println!("   📌 No bootstrap config found");
    }
    
    if bootstrap_connected == 0 {
        println!("   🌐 Using mDNS for local peer discovery only");
        println!("   💡 For mainnet: Set AXIOM_BOOTSTRAP_PEERS env var or update config/bootstrap.toml");
    } else {
        println!("   ✅ {} bootstrap nodes queued for connection", bootstrap_connected);
    }

    // Ask the network for peers' chains so we can self-heal/sync on startup
    let _ = swarm.behaviour_mut().gossipsub.publish(req_topic.clone(), b"REQ_CHAIN".to_vec());

    // 4. START OPENCLAW AUTOMATION (Background task for ceremony coordination & monitoring)
    println!("🤖 Initializing OpenClaw automation...");
    let _openclaw_handle = match openclaw_integration::start_openclaw_background().await {
        Ok(handle) => {
            println!("✅ OpenClaw started in background");
            Some(handle)
        },
        Err(e) => {
            println!("⚠️  OpenClaw startup warning: {}", e);
            None
        }
    };

    let mut last_vdf = Instant::now();
    let mut last_diff = tc.difficulty; // Initialization used here
    let mut last_bootstrap_retry = Instant::now();
    let mut vdf_loop = time::interval(Duration::from_millis(100));
    let mut dashboard_timer = time::interval(Duration::from_secs(10));
    let mut throttle_reset = time::interval(Duration::from_secs(60));
    let mut tx_broadcast_timer = time::interval(Duration::from_secs(30));
    let mut chain_sync_timer = time::interval(Duration::from_secs(300)); // Sync every 5 minutes
    let mut bootstrap_retry_timer = time::interval(Duration::from_secs(120)); // Retry bootstrap every 2 minutes
    let mut cross_network_discovery = time::interval(Duration::from_secs(30)); // Try cross-network peers every 30s
    
    // Track connected peers for network monitoring
    let mut connected_peers: std::collections::HashSet<libp2p::PeerId> = std::collections::HashSet::new();
    
    // Known peer addresses for cross-network discovery (can be set via env)
    let known_peers: Vec<String> = std::env::var("AXIOM_KNOWN_PEERS")
        .unwrap_or_default()
        .split(',')
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.trim().to_string())
        .collect();

    loop {
        tokio::select! {
            // --- P2P EVENT LOOP: AI-ASSISTED SPAM & DOS PROTECTION ---
            event = swarm.select_next_some() => match event {
                SwarmEvent::Behaviour(network::TimechainBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                    propagation_source, message, ..
                })) => {
                    // Rate limiting: allow max 100 messages per peer per minute
                    let now = Instant::now();
                    let entry = peer_message_counts.entry(propagation_source).or_insert((0, now));
                    if now.duration_since(entry.1) > Duration::from_secs(60) {
                        entry.0 = 0;
                        entry.1 = now;
                    }
                    entry.0 += 1;
                    if entry.0 > 100 {
                        println!("🚨 DoS protection: Peer {} exceeded message rate limit, ignoring", propagation_source);
                        continue;
                    }

                    let mut ai = ai_guardian.lock().unwrap();
                    let is_trustworthy = ai.predict_trust(1.0 / (entry.0 as f32), 1.0, 1.0);

                    if is_trustworthy && entry.0 <= 15 {
                        // 1) If this is a chain request, respond with our entire chain
                        if message.data == b"REQ_CHAIN" {
                            if let Ok(encoded) = bincode::serialize(&tc.blocks) {
                                let _ = swarm.behaviour_mut().gossipsub.publish(chain_topic.clone(), encoded);
                            }
                        }
                        // 2) If this is a block, validate and add it
                        else if message.topic == blocks_topic.hash() {
                            if let Ok(block) = bincode::deserialize::<Block>(&message.data) {
                                let elapsed = last_vdf.elapsed().as_secs();
                                if tc.add_block(block, elapsed).is_ok() {
                                    println!("✅ Block accepted and added to chain");
                                    storage::save_chain(&tc.blocks);
                                }
                            }
                        }
                        // 3) If this is a transaction, validate and add to mempool
                        else if message.topic == tx_topic.hash() {
                            if let Ok(tx) = bincode::deserialize::<Transaction>(&message.data) {
                                if tc.validate_transaction(&tx).is_ok() && !mempool.contains(&tx) {
                                    mempool.push_back(tx);
                                    println!("✅ Transaction added to mempool");
                                }
                            }
                        }

                        // 2) If this is a full chain broadcast, attempt to adopt it if it's longer and valid
                        else if message.topic == chain_topic.hash() {
                            if let Ok(peer_blocks) = bincode::deserialize::<Vec<Block>>(&message.data) {
                                // Enhanced chain validation for global consensus
                                if let Some(valid_chain) = validate_and_sync_chain(&peer_blocks, &tc) {
                                    tc = valid_chain;
                                    println!("🔁 Synced complete chain from peer. New height: {}", tc.blocks.len());
                                    storage::save_chain(&tc.blocks);
                                    last_vdf = Instant::now();

                                    // Broadcast our updated chain state to help other peers sync
                                    if let Ok(encoded) = bincode::serialize(&tc.blocks) {
                                        let _ = swarm.behaviour_mut().gossipsub.publish(chain_topic.clone(), encoded);
                                    }
                                }
                            }
                        }

                        // 3) Otherwise try to decode as a single Block (existing behaviour)
                        else if let Ok(incoming_block) = bincode::deserialize::<Block>(&message.data) {
                            let elapsed = last_vdf.elapsed().as_secs();

                            // RESOLVED: last_diff is now updated before being used in dashboard
                            last_diff = tc.difficulty;

                            if tc.add_block(incoming_block.clone(), elapsed).is_ok() {
                                println!("📥 AI Verified Block: H-{}", tc.blocks.len());
                                storage::save_chain(&tc.blocks);
                                last_vdf = Instant::now();
                                ai.train([1.0, 1.0, 1.0], 1.0);
                            }
                        }
                    } else if entry.0 > 20 {
                        ai.train([0.1, 0.0, 0.0], 0.0);
                    }
                },
                SwarmEvent::NewListenAddr { address, .. } => {
                    println!("🌐 Node active on: {:?}", address);
                    // Check if this is an external address
                    if address.to_string().contains("/ip4/") && !address.to_string().contains("/ip4/127.0.0.1") && !address.to_string().contains("/ip4/0.0.0.0") {
                        println!("🌍 External address detected! Other nodes can connect to: {}/p2p/{}", address, swarm.local_peer_id());
                    }
                    // Announce our current chain to the local network to help new peers sync
                    if let Ok(encoded) = bincode::serialize(&tc.blocks) {
                        let _ = swarm.behaviour_mut().gossipsub.publish(chain_topic.clone(), encoded);
                    }
                },
                SwarmEvent::ConnectionEstablished { peer_id, endpoint, .. } => {
                    connected_peers.insert(peer_id);
                    println!("🔗 Peer connected: {} | Total peers: {}", peer_id, connected_peers.len());
                    println!("   └─ Direction: {:?} | Address: {:?}", endpoint.is_dialer(), endpoint.get_remote_address());
                },
                SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
                    connected_peers.remove(&peer_id);
                    println!("🔌 Peer disconnected: {} | Total peers: {}", peer_id, connected_peers.len());
                    if let Some(err) = cause {
                        println!("   └─ Cause: {:?}", err);
                    }
                },

                // When mDNS discovers peers on the LAN, proactively request their chain
                SwarmEvent::Behaviour(network::TimechainBehaviourEvent::Mdns(ev)) => {
                    match ev {
                        libp2p::mdns::Event::Discovered(list) => {
                            for (peer_id, addr) in list {
                                println!("🔎 mDNS discovered peer: {} at {}", peer_id, addr);
                                // Actually dial the discovered peer to establish connection
                                if let Err(e) = swarm.dial(addr.clone()) {
                                    println!("   └─ ⚠️  Failed to dial: {:?}", e);
                                } else {
                                    println!("   └─ 📞 Dialing...");
                                }
                                let _ = swarm.behaviour_mut().gossipsub.publish(req_topic.clone(), b"REQ_CHAIN".to_vec());
                            }
                        }
                        libp2p::mdns::Event::Expired(list) => {
                            for (peer_id, _addr) in list {
                                println!("🔻 mDNS expired peer: {}", peer_id);
                            }
                        }
                    }
                },

                // When identify events occur (new peers), ask them for their chain
                SwarmEvent::Behaviour(network::TimechainBehaviourEvent::Identify(libp2p::identify::Event::Received { peer_id, info, .. })) => {
                    println!("👋 Identified peer: {} ({:?})", peer_id, info.agent_version);
                    let _ = swarm.behaviour_mut().gossipsub.publish(req_topic.clone(), b"REQ_CHAIN".to_vec());
                    // Also send a direct request-response asking for missing blocks
                    let _ = swarm.behaviour_mut().request_response.send_request(
                        &peer_id,
                        network::ChainRequest { start_height: tc.blocks.len() as u64 },
                    );
                },
                SwarmEvent::IncomingConnection { connection_id, local_addr, send_back_addr } => {
                    println!("📞 Incoming connection attempt from {}", send_back_addr);
                    println!("   └─ Local addr: {} | Connection ID: {:?}", local_addr, connection_id);
                },
                SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                    if let Some(peer) = peer_id {
                        println!("⚠️  Outgoing connection to {} failed: {:?}", peer, error);
                    } else {
                        println!("⚠️  Outgoing connection failed: {:?}", error);
                    }
                },
                SwarmEvent::IncomingConnectionError { send_back_addr, error, .. } => {
                    println!("⚠️  Incoming connection from {} failed: {:?}", send_back_addr, error);
                },
                SwarmEvent::Behaviour(network::TimechainBehaviourEvent::RequestResponse(ev)) => {
                    match ev {
                        libp2p::request_response::Event::Message { peer, message } => {
                            match message {
                                libp2p::request_response::Message::Request { request, channel, .. } => {
                                    // Peer asked for our chain starting at `start_height`
                                    let start = request.start_height as usize;
                                    let to_send = if start < tc.blocks.len() { tc.blocks[start..].to_vec() } else { Vec::new() };
                                    let resp = network::ChainResponse { blocks: to_send };
                                    let _ = swarm.behaviour_mut().request_response.send_response(channel, resp);
                                }
                                libp2p::request_response::Message::Response { response, .. } => {
                                    if !response.blocks.is_empty() {
                                        println!("📥 Received {} blocks via request-response from {}", response.blocks.len(), peer);
                                        for b in response.blocks {
                                            let _ = tc.add_block(b, 1800);
                                        }
                                        storage::save_chain(&tc.blocks);
                                    }
                                }
                            }
                        }
                        libp2p::request_response::Event::OutboundFailure { peer, error, .. } => {
                            // Fallback to gossipsub for any request-response failure
                            log::debug!("RequestResponse failure with peer {}: {:?} - using gossipsub fallback", peer, error);
                            let _ = swarm.behaviour_mut().gossipsub.publish(chain_topic.clone(), bincode::serialize(&tc.blocks).unwrap_or_default());
                        }
                        libp2p::request_response::Event::InboundFailure { peer, error, .. } => {
                            log::debug!("RequestResponse inbound failure from {}: {:?}", peer, error);
                        }
                        _ => {}
                    }
                },
                _ => {}
            },

            _ = throttle_reset.tick() => {
                peer_message_counts.clear();
            },

            // --- BROADCAST PENDING TRANSACTIONS ---
            _ = tx_broadcast_timer.tick() => {
                if let Ok(tx_data) = std::fs::read("pending_tx.dat") {
                    if let Ok(tx) = bincode::deserialize::<Transaction>(&tx_data) {
                        if tc.validate_transaction(&tx).is_ok() {
                            let encoded = bincode::serialize(&tx).unwrap();
                            let _ = swarm.behaviour_mut().gossipsub.publish(
                                gossipsub::IdentTopic::new("timechain-transactions"), encoded
                            );
                            println!("📤 Transaction broadcasted");
                            // Remove the pending transaction file
                            let _ = std::fs::remove_file("pending_tx.dat");
                        }
                    }
                }
            },

            // --- PERIODIC CHAIN SYNC: Ensure global consensus ---
            _ = chain_sync_timer.tick() => {
                println!("🔄 Performing periodic chain synchronization...");
                // Request chains from connected peers to ensure we're in sync
                let _ = swarm.behaviour_mut().gossipsub.publish(req_topic.clone(), b"REQ_CHAIN".to_vec());

                // Also request missing blocks via request-response if we detect gaps
                if connected_peers.len() > 0 {
                    let peer_ids: Vec<_> = connected_peers.iter().cloned().collect();
                    for peer_id in peer_ids {
                        let _ = swarm.behaviour_mut().request_response.send_request(
                            &peer_id,
                            network::ChainRequest { start_height: tc.blocks.len() as u64 },
                        );
                    }
                }

                // Broadcast our current chain state to help peers sync
                if let Ok(encoded) = bincode::serialize(&tc.blocks) {
                    let _ = swarm.behaviour_mut().gossipsub.publish(chain_topic.clone(), encoded);
                }
            },

            // --- DASHBOARD: RESOLVING UNUSED WARNINGS ---
            _ = dashboard_timer.tick() => {
                let elapsed = last_vdf.elapsed().as_secs();
                let remaining = 1800u64.saturating_sub(elapsed);
                // Using last_diff to calculate and show the difficulty trend
                let trend = if tc.difficulty > last_diff { "UP ⬆️" } else if tc.difficulty < last_diff { "DOWN ⬇️" } else { "STABLE ↔️" };
                // Supply info
                let (mined, remaining_supply, percent) = tc.supply_info();
                let mined_axm = Timechain::format_axm(mined);
                let remaining_axm = Timechain::format_axm(remaining_supply);
                println!("\n--- 🏛️  AXIOM STATUS ---");
                println!("⛓️  Height: {} | Diff: {} | Trend: {}", tc.blocks.len(), tc.difficulty, trend);
                println!("⏳ Time-Lock: {:02}m remaining | 🤖 AI Shield: ACTIVE", remaining/60);
                println!("💰 Mined: {} AXM | Remaining: {} AXM | {:.2}% of max supply", mined_axm, remaining_axm, percent);
                
                // --- ENHANCED NETWORK DIAGNOSTICS ---
                println!("🌐 Network Status:");
                println!("   ├─ PeerId: {}", swarm.local_peer_id());
                println!("   ├─ Connected Peers: {}", connected_peers.len());
                if connected_peers.is_empty() {
                    println!("   │  └─ No peers connected (check firewall/NAT)");
                } else {
                    for (i, peer) in connected_peers.iter().enumerate() {
                        let prefix = if i == connected_peers.len() - 1 { "   │  └─" } else { "   │  ├─" };
                        println!("{} {}", prefix, peer);
                    }
                }
                println!("   └─ Listen Addresses:");
                for addr in libp2p::Swarm::listeners(&swarm) {
                    println!("      └─ {}", addr);
                }
                
                // --- AI Dashboard Output ---
                let ai = ai_guardian.lock().unwrap();
                ai.log_stats();
                // Write stats to file for live monitor
                if let Ok(mut f) = std::fs::File::create("ai_stats.json") {
                    let _ = serde_json::to_writer_pretty(&mut f, &ai.stats);
                }
                println!("[Dashboard] AI stats written to ai_stats.json");
                println!("------------------------\n");
                // Sync last_diff for the next interval
                last_diff = tc.difficulty;
            },
            
            // --- BOOTSTRAP RETRY LOGIC ---
            _ = bootstrap_retry_timer.tick() => {
                // Periodically retry connecting to bootstrap nodes if not enough peers
                if connected_peers.len() < 2 && last_bootstrap_retry.elapsed().as_secs() > 120 {
                    log::info!("Retrying bootstrap connections (current peers: {})", connected_peers.len());
                    if let Ok(bootstrap_content) = std::fs::read_to_string("config/bootstrap.toml") {
                        if let Ok(bootstrap_config) = toml::from_str::<toml::Value>(&bootstrap_content) {
                            if let Some(bootnodes) = bootstrap_config.get("bootnodes").and_then(|v| v.as_array()) {
                                for bootnode in bootnodes {
                                    if let Some(addr_str) = bootnode.as_str() {
                                        if let Ok(addr) = addr_str.parse::<Multiaddr>() {
                                            let _ = swarm.dial(addr.clone());
                                        }
                                    }
                                }
                            }
                        }
                    }
                    last_bootstrap_retry = Instant::now();
                }
            },
            
            // --- CROSS-NETWORK PEER DISCOVERY ---
            _ = cross_network_discovery.tick() => {
                // Try connecting to known peers for cross-network scenarios
                if connected_peers.is_empty() && !known_peers.is_empty() {
                    for peer_addr in &known_peers {
                        if let Ok(addr) = peer_addr.parse::<Multiaddr>() {
                            log::debug!("Attempting cross-network connection to: {}", addr);
                            if let Err(e) = swarm.dial(addr.clone()) {
                                log::debug!("Cross-network dial attempt failed: {:?}", e);
                            }
                        }
                    }
                }
            },

            // --- MINING ENGINE ---
            _ = vdf_loop.tick() => {
                let elapsed = last_vdf.elapsed().as_secs();

                if elapsed >= 3600 {
                    let parent_hash = tc.blocks.last().unwrap().hash();
                    let current_slot = tc.blocks.len() as u64;
                    let vdf_seed = vdf::evaluate(parent_hash, current_slot);
                    let vdf_proof = compute_vdf(vdf_seed, tc.difficulty as u32);
                    let zk_pass = genesis::generate_zk_pass(&wallet, parent_hash);

                    // Select transactions from mempool (up to some limit)
                    let mut selected_txs = Vec::new();
                    let max_txs_per_block = 100;
                    while let Some(tx) = mempool.front() {
                        if selected_txs.len() >= max_txs_per_block {
                            break;
                        }
                        // Double-check transaction is still valid
                        if tc.validate_transaction(tx).is_ok() {
                            selected_txs.push(mempool.pop_front().unwrap());
                        } else {
                            // Remove invalid transaction
                            mempool.pop_front();
                        }
                    }

                    let mut nonce = 0u64;
                    let mut found = false;
                    let max_attempts = if tc.blocks.len() <= 2 {
                        1000000 // More attempts for early blocks
                    } else {
                        100000
                    };

                    while !found && nonce < max_attempts {
                        let candidate = Block {
                            parent: parent_hash,
                            slot: current_slot,
                            miner: wallet.address,
                            transactions: selected_txs.clone(),
                            vdf_proof,
                            zk_proof: zk_pass.clone(),
                            nonce,
                        };

                        if candidate.meets_difficulty(tc.difficulty)
                            && tc.add_block(candidate.clone(), elapsed).is_ok() {
                            println!("✨ MINED: H-{} | Nonce: {} | Txs: {}", tc.blocks.len(), nonce, selected_txs.len());
                            let encoded = bincode::serialize(&candidate).unwrap();
                            let _ = swarm.behaviour_mut().gossipsub.publish(
                                gossipsub::IdentTopic::new("timechain-blocks"), encoded
                            );
                            storage::save_chain(&tc.blocks);
                            last_vdf = Instant::now();
                            found = true;
                        }
                        nonce += 1;
                    }

                    // If mining failed, adjust difficulty for next attempt
                    if !found {
                        if tc.difficulty > 10 {
                            tc.difficulty = tc.difficulty.saturating_sub(10);
                            println!("⚠️  Mining failed, reducing difficulty to {}", tc.difficulty);
                        } else {
                            println!("⚠️  Mining failed at minimum difficulty. Check system performance.");
                      }
                    }
                }
            },
        }
    }
}
