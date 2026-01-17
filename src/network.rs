use libp2p::{gossipsub, mdns, kad, identify, swarm::{NetworkBehaviour, Swarm}};
use std::error::Error;
use libp2p::identity;
use libp2p::request_response::{self, ProtocolSupport};
use futures::io::{AsyncRead, AsyncWrite, AsyncReadExt, AsyncWriteExt};
use std::io;
use serde::{Serialize, Deserialize};
use crate::block::Block;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChainRequest { pub start_height: u64 }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChainResponse { pub blocks: Vec<Block> }

#[derive(Clone, Default)]
pub struct ChainCodec;

#[async_trait::async_trait]
impl request_response::Codec for ChainCodec {
    type Protocol = &'static str;
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
#[behaviour(out_event = "TimechainBehaviourEvent")]
pub struct TimechainBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub mdns: mdns::tokio::Behaviour,
    pub kademlia: kad::Behaviour<kad::store::MemoryStore>,
    pub identify: identify::Behaviour,
    pub request_response: request_response::Behaviour<ChainCodec>,
}

#[derive(Debug)]
pub enum TimechainBehaviourEvent {
    Gossipsub(gossipsub::Event),
    Mdns(mdns::Event),
    Kademlia(kad::Event),
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
impl From<kad::Event> for TimechainBehaviourEvent {
    fn from(event: kad::Event) -> Self { Self::Kademlia(event) }
}
impl From<identify::Event> for TimechainBehaviourEvent {
    fn from(event: identify::Event) -> Self { Self::Identify(event) }
}
impl From<request_response::Event<ChainRequest, ChainResponse>> for TimechainBehaviourEvent {
    fn from(event: request_response::Event<ChainRequest, ChainResponse>) -> Self { Self::RequestResponse(event) }
}

// Ensure this is PUB so main.rs can call it
pub async fn init_network() -> Result<Swarm<TimechainBehaviour>, Box<dyn Error + Send + Sync>> {
    let local_key = identity::Keypair::generate_ed25519();
    
    let swarm = libp2p::SwarmBuilder::with_existing_identity(local_key)
        .with_tokio()
        .with_tcp(libp2p::tcp::Config::default(), libp2p::noise::Config::new, libp2p::yamux::Config::default)?
        .with_behaviour(|key| {
            Ok(TimechainBehaviour {
                gossipsub: gossipsub::Behaviour::new(
                    gossipsub::MessageAuthenticity::Signed(key.clone()),
                    gossipsub::Config::default(),
                )?,
                mdns: mdns::tokio::Behaviour::new(mdns::Config::default(), key.public().to_peer_id())?,
                kademlia: kad::Behaviour::new(key.public().to_peer_id(), kad::store::MemoryStore::new(key.public().to_peer_id())),
                identify: identify::Behaviour::new(identify::Config::new("qubit/1.0.0".into(), key.public())),
                request_response: {
                    let cfg = request_response::Config::default();
                    request_response::Behaviour::new(vec![("/qubit/chain-sync/1.0.0", ProtocolSupport::Full)], cfg)
                },
            })
        })?
        .build();

    Ok(swarm)
}
