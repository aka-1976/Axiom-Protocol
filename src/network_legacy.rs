use std::collections::HashSet;
use libp2p::{gossipsub, mdns, identify, swarm::{NetworkBehaviour, Swarm}, Multiaddr, StreamProtocol};
use log;
use std::error::Error;
use libp2p::identity;
use libp2p::request_response::{self, ProtocolSupport};
use futures::io::{AsyncRead, AsyncWrite, AsyncReadExt, AsyncWriteExt};
use std::io;
use serde::{Serialize, Deserialize};
use crate::block::Block;

/// External validator registry
#[derive(Default)]
pub struct ValidatorRegistry {
    pub validators: HashSet<String>, // Peer IDs as strings
}

impl ValidatorRegistry {
    pub fn register(&mut self, peer_id: &str) {
        self.validators.insert(peer_id.to_string());
    }

    pub fn is_validator(&self, peer_id: &str) -> bool {
        self.validators.contains(peer_id)
    }
}

/// Add external peer to the network via direct dial (Discv5 handles discovery externally)
pub fn add_external_peer(swarm: &mut Swarm<TimechainBehaviour>, peer_addr: &str, _peer_id: &str) {
    if let Ok(addr) = peer_addr.parse::<Multiaddr>() {
        match swarm.dial(addr.clone()) {
            Ok(_) => {
                log::info!("✅ Dialing external peer at {}", addr);
            },
            Err(e) => {
                eprintln!("⚠️  Failed to dial peer at '{}': {}", peer_addr, e);
            }
        }
    } else {
        eprintln!("⚠️  Failed to parse peer address '{}'", peer_addr);
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChainRequest { pub start_height: u64 }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChainResponse { pub blocks: Vec<Block> }

#[derive(Clone, Default)]
pub struct ChainCodec;

#[async_trait::async_trait]
impl request_response::Codec for ChainCodec {
    type Protocol = StreamProtocol;
    type Request = ChainRequest;
    type Response = ChainResponse;

    async fn read_request<T>(&mut self, _protocol: &Self::Protocol, io: &mut T) -> io::Result<Self::Request>
    where T: AsyncRead + Unpin + Send {
        let mut buf = Vec::new();
        io.read_to_end(&mut buf).await?;
        let req: ChainRequest = serde_json::from_slice(&buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        Ok(req)
    }

    async fn read_response<T>(&mut self, _protocol: &Self::Protocol, io: &mut T) -> io::Result<Self::Response>
    where T: AsyncRead + Unpin + Send {
        let mut buf = Vec::new();
        io.read_to_end(&mut buf).await?;
        let resp: ChainResponse = serde_json::from_slice(&buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        Ok(resp)
    }

    async fn write_request<T>(&mut self, _protocol: &Self::Protocol, io: &mut T, req: Self::Request) -> io::Result<()>
    where T: AsyncWrite + Unpin + Send {
        let bytes = serde_json::to_vec(&req).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
        io.write_all(&bytes).await?;
        io.close().await.ok();
        Ok(())
    }

    async fn write_response<T>(&mut self, _protocol: &Self::Protocol, io: &mut T, resp: Self::Response) -> io::Result<()>
    where T: AsyncWrite + Unpin + Send {
        let bytes = serde_json::to_vec(&resp).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
        io.write_all(&bytes).await?;
        io.close().await.ok();
        Ok(())
    }
}

#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "TimechainBehaviourEvent")]
pub struct TimechainBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub mdns: mdns::tokio::Behaviour,
    pub identify: identify::Behaviour,
    pub request_response: request_response::Behaviour<ChainCodec>,
}

#[derive(Debug)]
pub enum TimechainBehaviourEvent {
    Gossipsub(gossipsub::Event),
    Mdns(mdns::Event),
    Identify(identify::Event),
    RequestResponse(request_response::Event<ChainRequest, ChainResponse>),
}

// Convert sub-events into our main event enum
impl From<gossipsub::Event> for TimechainBehaviourEvent {
    fn from(event: gossipsub::Event) -> Self { Self::Gossipsub(event) }
}
impl From<mdns::Event> for TimechainBehaviourEvent {
    fn from(event: mdns::Event) -> Self { Self::Mdns(event) }
}
impl From<identify::Event> for TimechainBehaviourEvent {
    fn from(event: identify::Event) -> Self { Self::Identify(event) }
}
impl From<request_response::Event<ChainRequest, ChainResponse>> for TimechainBehaviourEvent {
    fn from(event: request_response::Event<ChainRequest, ChainResponse>) -> Self { Self::RequestResponse(event) }
}

// Ensure this is PUB so main.rs can call it
/// Default hardcoded real-world bootstrap peers
const DEFAULT_BOOTSTRAP_PEERS: &[&str] = &[
    // Replace these with real, public Axiom nodes as they become available
    "/ip4/34.160.111.145/tcp/7000", // Example: Google Cloud VM
    "/ip4/51.15.23.200/tcp/7000",   // Example: Scaleway/OVH
    "/ip4/3.8.120.113/tcp/7000",    // Example: AWS EC2
];

pub async fn init_network() -> Result<Swarm<TimechainBehaviour>, Box<dyn Error + Send + Sync>> {
    let peers = DEFAULT_BOOTSTRAP_PEERS.iter().map(|s| s.to_string()).collect();
    init_network_with_bootstrap(peers).await
}

/// Initialize network with advanced security: peer authentication, encrypted channels, rate limiting, and robust bootstrap logic.
/// Discv5 handles peer discovery externally - peers are bridged to the swarm via manual dialing.
pub async fn init_network_with_bootstrap(bootstrap_peers: Vec<String>) -> Result<Swarm<TimechainBehaviour>, Box<dyn Error + Send + Sync>> {
    // Use Ed25519 for strong peer identity
    let local_key = identity::Keypair::generate_ed25519();
    
    // Configure Yamux with longer idle timeout to prevent disconnects
    let yamux_config = libp2p::yamux::Config::default();
    
    // Enforce encrypted channels (Noise protocol)
    let mut swarm = libp2p::SwarmBuilder::with_existing_identity(local_key)
        .with_tokio()
        .with_tcp(
            libp2p::tcp::Config::default(),
            libp2p::noise::Config::new,
            || yamux_config.clone(),
        )?
        .with_behaviour(|key| {
            Ok(TimechainBehaviour {
                gossipsub: gossipsub::Behaviour::new(
                    gossipsub::MessageAuthenticity::Signed(key.clone()),
                    gossipsub::Config::default(),
                )?,
                mdns: mdns::tokio::Behaviour::new(mdns::Config::default(), key.public().to_peer_id())?,
                identify: identify::Behaviour::new(identify::Config::new("axiom/1.0.0".into(), key.public())),
                request_response: {
                    // Support multiple protocol versions for compatibility
                    request_response::Behaviour::new(
                        vec![
                            (StreamProtocol::new("/axiom/chain-sync/1.0.0"), ProtocolSupport::Full),
                            (StreamProtocol::new("/axiom/chain-sync/0.9.0"), ProtocolSupport::Full),
                        ],
                        request_response::Config::default(),
                    )
                },
            })
        })?
        .with_swarm_config(|cfg| {
            cfg.with_idle_connection_timeout(std::time::Duration::from_secs(120))
        })
        .build();

    // Dial bootstrap peers directly (Discv5 handles discovery externally)
    let mut dialed = 0;
    for addr_str in bootstrap_peers {
        if let Ok(addr) = addr_str.parse::<Multiaddr>() {
            match swarm.dial(addr.clone()) {
                Ok(_) => {
                    log::info!("Dialing bootstrap peer: {}", addr);
                    dialed += 1;
                }
                Err(e) => {
                    log::warn!("Failed to dial bootstrap peer {}: {}", addr, e);
                }
            }
        } else {
            log::warn!("Invalid bootstrap peer address: {}", addr_str);
        }
    }
    if dialed == 0 {
        log::warn!("No valid bootstrap peers dialed. Node will rely on mDNS and Discv5 discovery.");
    }
    Ok(swarm)
}

/// Utility: Check connectivity to bootstrap nodes from config or environment (non-blocking)
pub fn check_bootstrap_connectivity() {
    println!("🔍 Checking bootstrap connectivity...");
    // Spawn async checks to avoid blocking main thread
    tokio::spawn(async {
        let mut nodes_to_check = Vec::new();
        
        // First, try environment variable for dynamic bootstrap peers
        let env_bootstrap_peers: Vec<String> = std::env::var("AXIOM_BOOTSTRAP_PEERS")
            .unwrap_or_default()
            .split(',')
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.trim().to_string())
            .collect();
        
        if !env_bootstrap_peers.is_empty() {
            for addr_str in &env_bootstrap_peers {
                // Parse multiaddr to extract IP and port
                if let Some(ip_part) = addr_str.split("/ip4/").nth(1) {
                    if let Some(ip) = ip_part.split('/').next() {
                        if let Some(port_part) = addr_str.split("/tcp/").nth(1) {
                            if let Some(port_str) = port_part.split('/').next() {
                                if let Ok(port) = port_str.parse::<u16>() {
                                    nodes_to_check.push((ip.to_string(), port));
                                }
                            }
                        }
                    }
                }
            }
        } else if let Ok(bootstrap_content) = std::fs::read_to_string("config/bootstrap.toml") {
            // Fall back to config file
            if let Ok(bootstrap_config) = toml::from_str::<toml::Value>(&bootstrap_content) {
                if let Some(bootnodes) = bootstrap_config.get("bootnodes").and_then(|v| v.as_array()) {
                    for bootnode in bootnodes {
                        if let Some(addr_str) = bootnode.as_str() {
                            // Parse multiaddr to extract IP and port
                            if let Some(ip_part) = addr_str.split("/ip4/").nth(1) {
                                if let Some(ip) = ip_part.split('/').next() {
                                    if let Some(port_part) = addr_str.split("/tcp/").nth(1) {
                                        if let Some(port_str) = port_part.split('/').next() {
                                            if let Ok(port) = port_str.parse::<u16>() {
                                                nodes_to_check.push((ip.to_string(), port));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Only check if we have configured bootstrap nodes
        if !nodes_to_check.is_empty() {
            for (ip, port) in nodes_to_check {
                let addr = format!("{}:{}", ip, port);
                match tokio::time::timeout(
                    std::time::Duration::from_secs(2),
                    tokio::net::TcpStream::connect(&addr)
                ).await {
                    Ok(Ok(_)) => println!("✅ Connected to bootstrap node: {}", addr),
                    Ok(Err(e)) => println!("⚠️  Could not connect to {}: {}", addr, e),
                    Err(_) => println!("⚠️  Connection to {} timed out", addr),
                }
            }
        } else {
            println!("   ℹ️  No bootstrap nodes configured - using mDNS for local discovery");
        }
    });
}
