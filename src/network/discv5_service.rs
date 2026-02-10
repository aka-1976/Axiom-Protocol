use discv5::{Discv5, enr::{CombinedKey, Enr, NodeId}, ConfigBuilder as Discv5ConfigBuilder, ListenConfig};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::Duration;

pub struct Discv5Service {
    discv5: Arc<RwLock<Discv5>>,
    local_enr: Enr<CombinedKey>,
}

impl Discv5Service {
    pub async fn new(
        listen_addr: SocketAddr,
        enr_key: CombinedKey,
        boot_nodes: Vec<Enr<CombinedKey>>,
    ) -> Result<Self, String> {
        let mut builder = Enr::builder();
        builder.ip(listen_addr.ip());
        builder.udp4(listen_addr.port());
        let local_enr = builder.build(&enr_key).map_err(|e| e.to_string())?;

        let listen_config = ListenConfig::from_ip(listen_addr.ip(), listen_addr.port());
        let config = Discv5ConfigBuilder::new(listen_config)
            .request_timeout(Duration::from_secs(10))
            .query_timeout(Duration::from_secs(30))
            .build();

        let mut discv5 = Discv5::new(local_enr.clone(), enr_key, config)
            .map_err(|e| e.to_string())?;
        
        for boot_enr in boot_nodes {
            if let Err(e) = discv5.add_enr(boot_enr.clone()) {
                log::warn!("Failed to add bootstrap node: {}", e);
            } else {
                log::info!("Added bootstrap node: {}", boot_enr.node_id());
            }
        }
        
        discv5.start().await.map_err(|e| e.to_string())?;
        log::info!("Discv5 discovery started on {}", listen_addr);
        
        Ok(Self {
            discv5: Arc::new(RwLock::new(discv5)),
            local_enr,
        })
    }
    
    pub async fn find_nodes(&self, target: NodeId) -> Vec<Enr<CombinedKey>> {
        let mut discv5 = self.discv5.write().await;
        match discv5.find_node(target).await {
            Ok(nodes) => nodes,
            Err(e) => {
                log::warn!("Find nodes error: {}", e);
                vec![]
            }
        }
    }
    
    pub fn local_enr(&self) -> Enr<CombinedKey> {
        self.local_enr.clone()
    }
    
    pub async fn connected_peers(&self) -> usize {
        let discv5 = self.discv5.read().await;
        discv5.connected_peers()
    }
    
    pub async fn table_entries(&self) -> Vec<Enr<CombinedKey>> {
        let discv5 = self.discv5.read().await;
        discv5.table_entries_enr()
    }
}

pub fn default_bootstrap_enrs() -> Vec<Enr<CombinedKey>> {
    vec![
        "enr:-IS4QHCYrYZbAKWCBRlAy5zzaDZXJBGkcnh4MHcBFZntXNFrdvJjX04jRzjzCBOonrkTfj499SZuOh8R33Ls8RRcy5wBgmlkgnY0gmlwhCIKrBSJc2VjcDI1NmsxoQPKY3i3_IJCdGaNADIwR0mO1n-bGx5RdVjbLLaFx0Y-koN0Y3CCfGODdWRwgnxj"
            .parse()
            .ok(),
    ]
    .into_iter()
    .flatten()
    .collect()
}