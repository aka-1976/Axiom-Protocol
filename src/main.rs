#![allow(unused_imports)]
#![allow(dead_code)]

mod block; mod transaction; mod chain; mod network; mod storage; 
mod main_helper; mod genesis; mod circuit; mod bridge; mod vdf; mod ai_engine;
mod state; mod economics; mod wallet;

use block::Block;
use chain::Timechain;
use transaction::Transaction;
use ai_engine::NeuralGuardian;
use main_helper::{Wallet, compute_vdf};
use libp2p::{gossipsub, swarm::SwarmEvent, futures::StreamExt, Multiaddr, PeerId};
use std::time::{Duration, Instant};
use tokio::time;
use std::error::Error;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("--------------------------------------------------");
    println!("🏛️  QUBIT CORE | DECENTRALIZED 84M PROTOCOL");
    println!("🛡️  STATUS: AI-NEURAL PROTECTION ACTIVE");
    println!("--------------------------------------------------");

    // 1. IDENTITY & STATE INITIALIZATION
    let wallet = Wallet::load_or_create();
    println!("💳 Wallet Address: {:?}", hex::encode(wallet.address));
    println!("📁 Wallet file: ./wallet.dat (keep safe!)");
    let ai_guardian = Arc::new(Mutex::new(NeuralGuardian::new()));
    let mut peer_message_counts: HashMap<PeerId, u32> = HashMap::new();

    // Transaction mempool
    let mut mempool: VecDeque<Transaction> = VecDeque::new();

    let mut tc = if let Some(saved_blocks) = storage::load_chain() {
        let mut chain = Timechain::new(genesis::genesis());
        for b in saved_blocks { let _ = chain.add_block(b, 3600); }
        chain
    } else {
        Timechain::new(genesis::genesis())
    };

    // 2. NETWORK SETUP
    // --- Network Setup with Dynamic Port Hunting ---
    let mut swarm = network::init_network().await?;

    let mut current_port: u16 = 6000;
    let max_port: u16 = 6010;

    loop {
        let addr: Multiaddr = format!("/ip4/0.0.0.0/tcp/{}", current_port).parse()?;
        match swarm.listen_on(addr.clone()) {
            Ok(_) => {
                println!("🌐 Node successfully bound to port: {}", current_port);
                break;
            }
            Err(e) => {
                if current_port < max_port {
                    println!("⚠️  Port {} busy. Trying {}...", current_port, current_port + 1);
                    current_port += 1;
                } else {
                    println!("❌ Critical Error: No available ports found in range 6000-6010.");
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

    // Ask the network for peers' chains so we can self-heal/sync on startup
    let _ = swarm.behaviour_mut().gossipsub.publish(req_topic.clone(), b"REQ_CHAIN".to_vec());

    let mut last_vdf = Instant::now();
    let mut last_diff = tc.difficulty; // Initialization used here
    let mut vdf_loop = time::interval(Duration::from_millis(100));
    let mut dashboard_timer = time::interval(Duration::from_secs(10));
    let mut throttle_reset = time::interval(Duration::from_secs(60));
    let mut tx_broadcast_timer = time::interval(Duration::from_secs(30));

    loop {
        tokio::select! {
            // --- P2P EVENT LOOP: AI-ASSISTED SPAM PROTECTION ---
            event = swarm.select_next_some() => match event {
                SwarmEvent::Behaviour(network::TimechainBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                    propagation_source, message, ..
                })) => {
                    let count = peer_message_counts.entry(propagation_source).or_insert(0);
                    *count += 1;

                    let mut ai = ai_guardian.lock().unwrap();
                    let is_trustworthy = ai.predict_trust(1.0 / (*count as f32), 1.0, 1.0);

                    if is_trustworthy && *count <= 15 {
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

                        // 2) If this is a full chain broadcast, attempt to adopt it if it's longer
                        else if let Ok(peer_blocks) = bincode::deserialize::<Vec<Block>>(&message.data) {
                            // Try to reconstruct a Timechain from the peer's blocks
                            let mut candidate = Timechain::new(genesis::genesis());
                            let mut valid = true;
                            for b in peer_blocks.iter().skip(1) {
                                if candidate.add_block(b.clone(), 3600).is_err() {
                                    valid = false;
                                    break;
                                }
                            }
                            if valid && candidate.blocks.len() > tc.blocks.len() {
                                tc = candidate;
                                println!("🔁 Synced chain from peer. New height: {}", tc.blocks.len());
                                storage::save_chain(&tc.blocks);
                                last_vdf = Instant::now();
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
                    } else if *count > 20 {
                        ai.train([0.1, 0.0, 0.0], 0.0);
                    }
                },
                SwarmEvent::NewListenAddr { address, .. } => {
                    println!("🌐 Node active on: {:?}", address);
                    // Announce our current chain to the local network to help new peers sync
                    if let Ok(encoded) = bincode::serialize(&tc.blocks) {
                        let _ = swarm.behaviour_mut().gossipsub.publish(chain_topic.clone(), encoded);
                    }
                },

                // When mDNS discovers peers on the LAN, proactively request their chain
                SwarmEvent::Behaviour(network::TimechainBehaviourEvent::Mdns(ev)) => {
                    match ev {
                        libp2p::mdns::Event::Discovered(list) => {
                            for (peer_id, _addr) in list {
                                println!("🔎 mDNS discovered peer: {}", peer_id);
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
                SwarmEvent::Behaviour(network::TimechainBehaviourEvent::Identify(libp2p::identify::Event::Received { peer_id, info })) => {
                    println!("👋 Identified peer: {} ({:?})", peer_id, info.agent_version);
                    let _ = swarm.behaviour_mut().gossipsub.publish(req_topic.clone(), b"REQ_CHAIN".to_vec());
                    // Also send a direct request-response asking for missing blocks
                    let _ = swarm.behaviour_mut().request_response.send_request(
                        &peer_id,
                        network::ChainRequest { start_height: tc.blocks.len() as u64 },
                    );
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
                                            let _ = tc.add_block(b, 3600);
                                        }
                                        storage::save_chain(&tc.blocks);
                                    }
                                }
                            }
                        }
                        libp2p::request_response::Event::OutboundFailure { peer, error, .. } => {
                            eprintln!("RequestResponse outbound failure to {}: {:?}", peer, error);
                        }
                        libp2p::request_response::Event::InboundFailure { peer, error, .. } => {
                            eprintln!("RequestResponse inbound failure from {}: {:?}", peer, error);
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

            // --- DASHBOARD: RESOLVING UNUSED WARNINGS ---
            _ = dashboard_timer.tick() => {
                let elapsed = last_vdf.elapsed().as_secs();
                let remaining = 3600u64.saturating_sub(elapsed);
                
                // Using last_diff to calculate and show the difficulty trend
                let trend = if tc.difficulty > last_diff { "UP ⬆️" } else if tc.difficulty < last_diff { "DOWN ⬇️" } else { "STABLE ↔️" };

                // Supply info
                let (mined, remaining_supply, percent) = tc.supply_info();
                let mined_qbt = Timechain::format_qbt(mined);
                let remaining_qbt = Timechain::format_qbt(remaining_supply);

                println!("\n--- 🏛️  QUBIT STATUS ---");
                println!("⛓️  Height: {} | Diff: {} | Trend: {}", tc.blocks.len(), tc.difficulty, trend);
                println!("⏳ Time-Lock: {:02}m remaining | 🤖 AI Shield: ACTIVE", remaining/60);
                println!("💰 Mined: {} QBT | Remaining: {} QBT | {:.2}% of max supply", mined_qbt, remaining_qbt, percent);
                println!("------------------------\n");
                
                // Sync last_diff for the next interval
                last_diff = tc.difficulty;
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

                    while !found && nonce < 100000 {
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
                }
            }
        }
    }
}
