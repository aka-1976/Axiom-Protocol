# Release v3.1.0 - Discv5 Network & Modular Upgrade

## Summary
This release upgrades AXIOM Protocol from Blake3-based networking to a production-grade Discv5/libp2p stack, with modular network services, metrics, orchestrator, and integration tests. All changes are mainnet-ready and fully documented.

## Major Changes
- **Discv5/libp2p Networking Stack**: Ethereum-style peer discovery, ENR, Gossipsub, Kademlia, mDNS, Identify, Ping
- **Modular Network Services**: `src/network/config.rs`, `discv5_service.rs`, `peer_manager.rs`, `behaviour.rs`, `gossip_handler.rs`, `event_handler.rs`, `mod.rs`
- **Peer Manager**: Reputation, LRU cache, ban logic, metrics
- **Gossip Handler**: Block/tx/sync/heartbeat routing, deduplication
- **Metrics Module**: Prometheus-style node metrics (`src/metrics/mod.rs`)
- **Main Orchestrator**: `src/main.rs` integrates network, metrics, CLI
- **Integration Tests**: `tests/integration_test.rs` for network stack validation
- **Legacy Network**: Old code moved to `src/network_legacy.rs`
- **Cargo.toml**: Added dashmap, lru, sha3 dependencies
- **README.md**: Updated with upgrade summary and usage

## Deployment & Validation
- All modules compile cleanly (no errors/warnings)
- Integration tests included and passing
- PRs pushed to Ghost-84M/Axiom-Protocol main branch
- Ready for mainnet deployment and monitoring

## Upgrade Instructions
- Build and run as before; node now uses new modular network stack
- See `src/network/` for new network code
- See `src/metrics/` for metrics collection
- See `tests/integration_test.rs` for integration test example

## Contributors
- Ghost-84M
- khanssameer19-png

## Date
February 7, 2026
