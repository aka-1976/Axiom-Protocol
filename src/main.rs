#![allow(unused_imports)]
#![allow(dead_code)]

mod block; mod transaction; mod chain; mod network; mod storage; 
mod main_helper; mod genesis; mod circuit; mod bridge; mod vdf; mod ai_engine;

use block::Block;
use chain::Timechain;
use ai_engine::NeuralGuardian;
use main_helper::{Wallet, compute_vdf};
use libp2p::{gossipsub, swarm::SwarmEvent, futures::StreamExt, Multiaddr, PeerId};
use std::time::{Duration, Instant};
use tokio::time;
use std::error::Error;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("--------------------------------------------------");
    println!("🏛️  QUBIT CORE | DECENTRALIZED 84M PROTOCOL");
    println!("🛡️  STATUS: AI-NEURAL PROTECTION ACTIVE");
    println!("--------------------------------------------------");

    // 1. IDENTITY & STATE INITIALIZATION
    let wallet = Wallet::load_or_create();
    let ai_guardian = Arc::new(Mutex::new(NeuralGuardian::new()));
    let mut peer_message_counts: HashMap<PeerId, u32> = HashMap::new();

    let mut tc = if let Some(saved_blocks) = storage::load_chain() {
        let mut chain = Timechain::new(genesis::genesis());
        for b in saved_blocks { let _ = chain.add_block(b, 3600); }
        chain
    } else {
        Timechain::new(genesis::genesis())
    };

    // 2. NETWORK SETUP
    let mut swarm = network::init_network().await?;
    swarm.listen_on("/ip4/0.0.0.0/tcp/6000".parse()?)?;

    let mut last_vdf = Instant::now();
    let mut last_diff = tc.difficulty; // Initialization used here
    let mut vdf_loop = time::interval(Duration::from_millis(100));
    let mut dashboard_timer = time::interval(Duration::from_secs(10));
    let mut throttle_reset = time::interval(Duration::from_secs(60));

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
                        if let Ok(incoming_block) = bincode::deserialize::<Block>(&message.data) {
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
                },
                _ => {}
            },

            _ = throttle_reset.tick() => {
                peer_message_counts.clear();
            },

            // --- DASHBOARD: RESOLVING UNUSED WARNINGS ---
            _ = dashboard_timer.tick() => {
                let elapsed = last_vdf.elapsed().as_secs();
                let remaining = if elapsed < 3600 { 3600 - elapsed } else { 0 };
                
                // Using last_diff to calculate and show the difficulty trend
                let trend = if tc.difficulty > last_diff { "UP ⬆️" } else if tc.difficulty < last_diff { "DOWN ⬇️" } else { "STABLE ↔️" };

                println!("\n--- 🏛️  QUBIT STATUS ---");
                println!("⛓️  Height: {} | Diff: {} | Trend: {}", tc.blocks.len(), tc.difficulty, trend);
                println!("⏳ Time-Lock: {:02}m remaining | 🤖 AI Shield: ACTIVE", remaining/60);
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

                    let mut nonce = 0u64;
                    let mut found = false;

                    while !found && nonce < 100000 {
                        let candidate = Block {
                            parent: parent_hash,
                            slot: current_slot,
                            miner: wallet.address,
                            transactions: vec![],
                            vdf_proof,
                            zk_proof: zk_pass.clone(),
                            nonce,
                        };

                        if candidate.meets_difficulty(tc.difficulty) {
                            if tc.add_block(candidate.clone(), elapsed).is_ok() {
                                println!("✨ MINED: H-{} | Nonce: {}", tc.blocks.len(), nonce);
                                let encoded = bincode::serialize(&candidate).unwrap();
                                let _ = swarm.behaviour_mut().gossipsub.publish(
                                    gossipsub::IdentTopic::new("timechain-blocks"), encoded
                                );
                                storage::save_chain(&tc.blocks);
                                last_vdf = Instant::now();
                                found = true;
                            }
                        }
                        nonce += 1;
                    }
                }
            }
        }
    }
}
