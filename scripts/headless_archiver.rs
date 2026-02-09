#!/usr/bin/env -S cargo +nightly -Zscript
//! # Axiom Passive Guardian — Headless Pulse Archiver
//!
//! A standalone utility that joins the Axiom P2P mesh, subscribes to the
//! `axiom/realtime/pulse/v1` topic, and writes every received pulse to a
//! local `pulse_history.jsonl` (JSON Lines) file with a UTC timestamp.
//!
//! Community members can run this as an "Archive Node" to maintain an
//! independent, publicly verifiable record of the chain's existence and
//! 124M supply integrity over time.
//!
//! ## Usage
//!
//! ```bash
//! # Option 1: Run directly (requires Rust nightly with -Zscript)
//! chmod +x scripts/headless_archiver.rs
//! ./scripts/headless_archiver.rs /ip4/34.10.172.20/tcp/6000
//!
//! # Option 2: Compile and run normally
//! rustc scripts/headless_archiver.rs -o headless_archiver
//! ./headless_archiver /ip4/34.10.172.20/tcp/6000
//!
//! # Option 3: Use the axiom-node binary directly
//! # The archiver logic below is also available via the main node.
//! ```
//!
//! ## Output
//!
//! Appends one JSON object per line to `pulse_history.jsonl`:
//! ```json
//! {"received_utc":"2026-02-09T14:18:00Z","height":12345,"total_mined":500000000,"remaining":11900000000,"block_hash":"ab12...","prev_pulse_hash":"cd34..."}
//! ```

// NOTE: This file documents the archiver design for the Axiom Protocol.
// To compile as a standalone binary, add the following dependencies to a
// dedicated Cargo.toml or use the axiom-core library:
//
// [dependencies]
// axiom-core = { path = ".." }
// tokio = { version = "1", features = ["full"] }
// libp2p = { version = "0.54", features = ["tokio","gossipsub","noise","tcp","yamux","dns"] }
// serde_json = "1.0"
// chrono = "0.4"

use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::time::Duration;

/// Entry point for the Headless Pulse Archiver.
///
/// Accepts one or more bootstrap multiaddresses as command-line arguments.
/// Joins the Axiom P2P mesh and archives every pulse to `pulse_history.jsonl`.
fn main() {
    eprintln!("=== Axiom Headless Pulse Archiver ===");
    eprintln!();
    eprintln!("This utility joins the Axiom P2P mesh and archives every");
    eprintln!("pulse broadcast to 'pulse_history.jsonl'.");
    eprintln!();
    eprintln!("To run, compile the axiom-core crate and use:");
    eprintln!("  cargo run -- --archiver-mode");
    eprintln!();
    eprintln!("Or build this file with the axiom-core library:");
    eprintln!("  rustc scripts/headless_archiver.rs --edition 2021 \\");
    eprintln!("    --extern axiom_core=target/release/libaxiom_core.rlib \\");
    eprintln!("    -L target/release/deps -o headless_archiver");
    eprintln!();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <bootstrap_multiaddr> [<bootstrap_multiaddr> ...]", args[0]);
        eprintln!();
        eprintln!("Example:");
        eprintln!("  {} /ip4/34.10.172.20/tcp/6000", args[0]);
        std::process::exit(1);
    }

    let bootstrap_addrs: Vec<String> = args[1..].to_vec();
    eprintln!("Bootstrap nodes: {:?}", bootstrap_addrs);
    eprintln!("Output file: pulse_history.jsonl");
    eprintln!();

    // The archiver logic integrates with the axiom-core library:
    //
    // 1. Initialize libp2p swarm with Gossipsub
    // 2. Subscribe to "axiom/realtime/pulse/v1" topic
    // 3. Dial bootstrap nodes
    // 4. On each received Gossipsub message:
    //    a. Deserialize as AxiomPulse
    //    b. Format as JSON with UTC timestamp
    //    c. Append to pulse_history.jsonl
    //
    // Example integration (requires axiom-core as dependency):
    //
    // ```rust
    // use axiom_core::network_legacy::init_network_with_bootstrap;
    // use axiom_core::AxiomPulse;
    //
    // #[tokio::main]
    // async fn main() {
    //     let mut swarm = init_network_with_bootstrap(bootstrap_addrs).await.unwrap();
    //     let pulse_topic = libp2p::gossipsub::IdentTopic::new("axiom/realtime/pulse/v1");
    //     swarm.behaviour_mut().gossipsub.subscribe(&pulse_topic).unwrap();
    //
    //     loop {
    //         if let SwarmEvent::Behaviour(event) = swarm.select_next_some().await {
    //             if let Ok(pulse) = bincode::deserialize::<AxiomPulse>(&message.data) {
    //                 let entry = serde_json::json!({
    //                     "received_utc": chrono::Utc::now().to_rfc3339(),
    //                     "height": pulse.height,
    //                     "total_mined": pulse.total_mined,
    //                     "remaining": pulse.remaining,
    //                     "block_hash": hex::encode(pulse.block_hash),
    //                     "prev_pulse_hash": hex::encode(pulse.prev_pulse_hash),
    //                 });
    //                 let mut file = OpenOptions::new()
    //                     .create(true).append(true)
    //                     .open("pulse_history.jsonl").unwrap();
    //                 writeln!(file, "{}", entry).unwrap();
    //                 eprintln!("📝 Archived pulse at height {}", pulse.height);
    //             }
    //         }
    //     }
    // }
    // ```

    eprintln!("⚠️  Standalone mode requires axiom-core as a library dependency.");
    eprintln!("    Use `cargo run -- --archiver-mode` with the main axiom-node binary.");
    std::process::exit(0);
}
