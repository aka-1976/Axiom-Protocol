use libp2p::gossipsub::{Behaviour as Gossipsub, Event as GossipsubEvent, IdentTopic, MessageAuthenticity, Config as GossipsubConfig};
use libp2p::identify::{Behaviour as Identify, Config as IdentifyConfig, Event as IdentifyEvent};
use libp2p::swarm::NetworkBehaviour;
use libp2p::identity::Keypair;
use libp2p_swarm_derive::NetworkBehaviour;

#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "AxiomEvent")]
pub struct AxiomBehaviour {
    pub gossipsub: Gossipsub,
    pub identify: Identify,
}

#[derive(Debug)]
pub enum AxiomEvent {
    Gossipsub(GossipsubEvent),
    Identify(IdentifyEvent),
}

impl From<GossipsubEvent> for AxiomEvent {
    fn from(event: GossipsubEvent) -> Self {
        AxiomEvent::Gossipsub(event)
    }
}

impl From<IdentifyEvent> for AxiomEvent {
    fn from(event: IdentifyEvent) -> Self {
        AxiomEvent::Identify(event)
    }
}

impl AxiomBehaviour {
    pub fn new(keypair: &Keypair) -> Result<Self, Box<dyn std::error::Error>> {
        let gossipsub_config = GossipsubConfig::default();
        let gossipsub = Gossipsub::new(MessageAuthenticity::Signed(keypair.clone()), gossipsub_config)?;
        let identify = Identify::new(IdentifyConfig::new("/axiom/1.0.0".to_string(), keypair.public()));

        Ok(Self { gossipsub, identify })
    }

    pub fn subscribe_to_topic(&mut self, topic_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let topic = IdentTopic::new(topic_name);
        self.gossipsub.subscribe(&topic)?;
        Ok(())
    }

    pub fn publish_message(&mut self, topic_name: &str, data: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
        let topic = IdentTopic::new(topic_name);
        self.gossipsub.publish(topic, data)?;
        Ok(())
    }

    pub fn connected_peers(&self) -> usize {
        self.gossipsub.all_peers().count()
    }
}
