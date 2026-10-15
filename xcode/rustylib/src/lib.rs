uniffi::setup_scaffolding!();

use rust_ipfs::builder::DefaultIpfsBuilder as IpfsBuilder;
use rust_ipfs::{Ipfs, IpfsPath};
use rust_ipfs::unixfs::UnixfsStatus;
use std::sync::Arc;

#[derive(Debug, uniffi::Error)]
pub enum IpfsError {
    Message(String),
}

impl std::fmt::Display for IpfsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IpfsError::Message(message) => write!(f, "{message}"),
        }
    }
}

#[uniffi::export]
fn rust_hello() -> String {
    "Hello from Rust!".to_string()
}

#[uniffi::export]
pub fn rust_add(a: u32, b: u32) -> u32 {
    a + b
}

#[derive(uniffi::Object)]
pub struct IpfsNode {
    inner: Ipfs,
    rt: tokio::runtime::Runtime,
    pubsub_storage:
        std::sync::Arc<std::sync::Mutex<std::collections::HashMap<String, std::collections::VecDeque<(String, Vec<u8>)>>>>,
}

#[uniffi::export]
impl IpfsNode {
    #[uniffi::constructor]
    pub fn new() -> Arc<Self> {
        let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
        let ipfs: Ipfs = rt.block_on(async {
            IpfsBuilder::new()
                .with_default()
                .enable_tcp()
                .enable_dns()
                .set_default_listener()
                .start()
                .await
                .expect("failed to start ipfs node")
        });
        Arc::new(Self {
            inner: ipfs,
            rt,
            pubsub_storage: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::HashMap::new(),
            )),
        })
    }

    pub fn pubsub_subscribe(&self, topic: String) -> String {
        use futures::StreamExt;

        self.pubsub_storage.lock().unwrap().entry(topic.clone()).or_default();
        let storage = self.pubsub_storage.clone();
        let node = self.inner.clone();
        let topic_for_task = topic.clone();

        self.rt.spawn(async move {
            if let Ok(mut stream) = node.pubsub_listener(topic_for_task.clone()).await {
                while let Some(event) = stream.next().await {
                    if let connexa::prelude::GossipsubEvent::Message { message } = event {
                        let from = message
                            .source
                            .map(|peer| peer.to_string())
                            .unwrap_or_default();
                        let mut guard = storage.lock().unwrap();
                        guard
                            .entry(topic_for_task.clone())
                            .or_default()
                            .push_back((from, message.data.to_vec()));
                    }
                }
            }
        });

        self.rt.block_on(async {
            match self.inner.pubsub_subscribe(topic).await {
                Ok(()) => "subscribed".to_string(),
                Err(err) => format!("subscribe failed: {err}"),
            }
        })
    }

    pub fn pubsub_unsubscribe(&self, topic: String) -> String {
        self.rt.block_on(async {
            match self.inner.pubsub_unsubscribe(topic).await {
                Ok(()) => "unsubscribed".to_string(),
                Err(err) => format!("unsubscribe failed: {err}"),
            }
        })
    }

    pub fn pubsub_publish(&self, topic: String, data: Vec<u8>) -> String {
        self.rt.block_on(async {
            match self.inner.pubsub_publish(topic, data).await {
                Ok(()) => "published".to_string(),
                Err(err) => format!("publish failed: {err}"),
            }
        })
    }

    pub fn pubsub_messages(&self, topic: String) -> Vec<String> {
        let mut storage = self.pubsub_storage.lock().unwrap();
        storage
            .get_mut(&topic)
            .map(|queue| {
                queue
                    .drain(..)
                    .map(|(from, data)| {
                        format!(
                            "{}: {}",
                            from,
                            data.iter().map(|byte| format!("{byte:02x}")).collect::<String>()
                        )
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn pubsub_peers(&self, topic: String) -> Vec<String> {
        self.rt.block_on(async {
            self.inner
                .pubsub_peers(topic)
                .await
                .map(|peers| peers.into_iter().map(|peer| peer.to_string()).collect())
                .unwrap_or_default()
        })
    }

    pub fn peer_id(&self) -> String {
        self.rt.block_on(async {
            match self.inner.identity(None).await {
                Ok(peer) => peer.peer_id.to_string(),
                Err(err) => format!("failed to get identity: {err}"),
            }
        })
    }

    pub fn listening_addresses(&self) -> Vec<String> {
        self.rt.block_on(async {
            self.inner
                .listening_addresses()
                .await
                .map(|addresses| addresses.into_iter().map(|address| address.to_string()).collect())
                .unwrap_or_default()
        })
    }

    pub fn addrs(&self) -> Vec<String> {
        self.rt.block_on(async {
            self.inner
                .addrs()
                .await
                .map(|peers| {
                    peers
                        .into_iter()
                        .flat_map(|(_, addresses)| addresses)
                        .map(|address| address.to_string())
                        .collect()
                })
                .unwrap_or_default()
        })
    }

    pub fn default_bootstrap(&self) -> String {
        self.rt.block_on(async {
            match self.inner.default_bootstrap().await {
                Ok(addresses) => addresses.into_iter().map(|address| address.to_string()).collect::<Vec<_>>().join("\n"),
                Err(err) => format!("failed to bootstrap: {err}"),
            }
        })
    }

    pub fn bootstraps(&self) -> Vec<String> {
        self.rt.block_on(async {
            self.inner
                .get_bootstraps()
                .await
                .map(|addresses| addresses.into_iter().map(|address| address.to_string()).collect())
                .unwrap_or_default()
        })
    }

    pub fn connect(&self, address: String) -> String {
        use std::str::FromStr;
        let addr = match rust_ipfs::Multiaddr::from_str(&address) {
            Ok(addr) => addr,
            Err(err) => return format!("invalid multiaddr: {err}"),
        };
        self.rt.block_on(async {
            match self.inner.connect(addr).await {
                Ok(id) => format!("connected: {id}"),
                Err(err) => format!("connect failed: {err}"),
            }
        })
    }

    pub fn mfs_mkdir(&self, path: String, parents: bool) -> String {
        self.rt.block_on(async {
            match self.inner.mfs().mkdir(&path, parents).await {
                Ok(()) => "ok".to_string(),
                Err(err) => format!("mkdir failed: {err}"),
            }
        })
    }

    pub fn mfs_write(&self, path: String, data: Vec<u8>, parents: bool) -> String {
        self.rt.block_on(async {
            match self.inner.mfs().write(&path, &data, parents).await {
                Ok(()) => "ok".to_string(),
                Err(err) => format!("write failed: {err}"),
            }
        })
    }

    pub fn mfs_read(&self, path: String) -> Result<Vec<u8>, IpfsError> {
        self.rt.block_on(async {
            self.inner
                .mfs()
                .read(&path)
                .await
                .map_err(|err| IpfsError::Message(format!("read failed: {err}")))
        })
    }

    pub fn mfs_rm(&self, path: String, recursive: bool) -> String {
        self.rt.block_on(async {
            match self.inner.mfs().rm(&path, recursive).await {
                Ok(()) => "ok".to_string(),
                Err(err) => format!("rm failed: {err}"),
            }
        })
    }

    pub fn mfs_ls(&self, path: String) -> Vec<String> {
        self.rt.block_on(async {
            self.inner
                .mfs()
                .ls(&path)
                .await
                .map(|entries| {
                    entries
                        .into_iter()
                        .map(|entry| match entry.kind {
                            rust_ipfs::mfs::MfsKind::File { size } => format!("{} file {} {}", entry.name, entry.cid, size),
                            rust_ipfs::mfs::MfsKind::Directory => format!("{} dir {}", entry.name, entry.cid),
                            rust_ipfs::mfs::MfsKind::Symlink => format!("{} symlink {}", entry.name, entry.cid),
                        })
                        .collect()
                })
                .unwrap_or_default()
        })
    }

    pub fn peers(&self) -> Vec<String> {
        self.rt.block_on(async {
            self.inner
                .connected()
                .await
                .map(|peers| peers.into_iter().map(|peer| peer.to_string()).collect())
                .unwrap_or_default()
        })
    }

    pub fn disconnect(&self, peer_id: String) -> String {
        use std::str::FromStr;
        let peer = match rust_ipfs::PeerId::from_str(&peer_id) {
            Ok(peer) => peer,
            Err(err) => return format!("invalid peer id: {err}"),
        };
        self.rt.block_on(async {
            match self.inner.disconnect(peer).await {
                Ok(()) => "disconnected".to_string(),
                Err(err) => format!("disconnect failed: {err}"),
            }
        })
    }

    pub fn external_addresses(&self) -> Vec<String> {
        self.rt.block_on(async {
            self.inner
                .external_addresses()
                .await
                .map(|addresses| addresses.into_iter().map(|address| address.to_string()).collect())
                .unwrap_or_default()
        })
    }

    pub fn add_external_address(&self, address: String) -> String {
        use std::str::FromStr;
        let addr = match rust_ipfs::Multiaddr::from_str(&address) {
            Ok(addr) => addr,
            Err(err) => return format!("invalid multiaddr: {err}"),
        };
        self.rt.block_on(async {
            match self.inner.add_external_address(addr).await {
                Ok(()) => "ok".to_string(),
                Err(err) => format!("failed: {err}"),
            }
        })
    }

    pub fn remove_external_address(&self, address: String) -> String {
        use std::str::FromStr;
        let addr = match rust_ipfs::Multiaddr::from_str(&address) {
            Ok(addr) => addr,
            Err(err) => return format!("invalid multiaddr: {err}"),
        };
        self.rt.block_on(async {
            match self.inner.remove_external_address(addr).await {
                Ok(()) => "ok".to_string(),
                Err(err) => format!("failed: {err}"),
            }
        })
    }

    pub fn find_peer(&self, peer_id: String) -> Vec<String> {
        use std::str::FromStr;
        let peer = match rust_ipfs::PeerId::from_str(&peer_id) {
            Ok(peer) => peer,
            Err(_) => return Vec::new(),
        };
        self.rt.block_on(async {
            self.inner
                .find_peer(peer)
                .await
                .map(|addresses| addresses.into_iter().map(|address| address.to_string()).collect())
                .unwrap_or_default()
        })
    }

    pub fn refs_local(&self) -> Vec<String> {
        self.rt.block_on(async { self.inner.refs_local().await.into_iter().map(|cid| cid.to_string()).collect() })
    }

    pub fn mfs_root(&self) -> String {
        self.rt.block_on(async {
            match self.inner.mfs().root().await {
                Ok(Some(cid)) => cid.to_string(),
                Ok(None) => "empty".to_string(),
                Err(err) => format!("root failed: {err}"),
            }
        })
    }

    pub fn identity_json(&self) -> String {
        self.rt.block_on(async {
            match self.inner.identity(None).await {
                Ok(peer) => format!(
                    "peer_id: {}\nprotocol_version: {}\nagent_version: {}\nlisten_addrs: {}",
                    peer.peer_id,
                    peer.protocol_version,
                    peer.agent_version,
                    peer.listen_addrs.into_iter().map(|a| a.to_string()).collect::<Vec<_>>().join(", ")
                ),
                Err(err) => format!("failed to get identity: {err}"),
            }
        })
    }

    pub fn add_bytes(&self, name: String, data: Vec<u8>) -> Result<String, IpfsError> {
        use futures::StreamExt;

        let stream = futures::stream::once(async move { Ok::<_, std::io::Error>(data) }).boxed();
        let mut add = self.inner.unixfs().add((name, stream));

        let path = self.rt.block_on(async {
            loop {
                match add.next().await {
                    Some(UnixfsStatus::CompletedStatus { path, .. }) => return Ok(path.to_string()),
                    Some(UnixfsStatus::FailedStatus { error, .. }) => {
                        return Err(IpfsError::Message(format!("failed to add bytes: {error}")));
                    }
                    Some(UnixfsStatus::ProgressStatus { .. }) => {}
                    None => return Err(IpfsError::Message("add completed without a path".to_string()))
                }
            }
        });

        path
    }

    pub fn cat(&self, path: String) -> Result<Vec<u8>, IpfsError> {
        use futures::StreamExt;

        let parsed: IpfsPath = path.parse().map_err(|err| IpfsError::Message(format!("invalid ipfs path: {err}")))?;
        let mut cat = self.inner.cat_unixfs(parsed);

        self.rt.block_on(async {
            let mut output = Vec::new();
            loop {
                match cat.next().await {
                    Some(Ok(bytes)) => output.extend_from_slice(&bytes),
                    Some(Err(err)) => return Err(IpfsError::Message(format!("failed to cat: {err}"))),
                    None => break,
                }
            }
            Ok(output)
        })
    }
}

#[uniffi::export]
pub fn start_ipfs_node() -> String {
    let node = IpfsNode::new();
    node.peer_id()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_starts() {
        let node = IpfsNode::new();
        assert!(!node.peer_id().is_empty());
    }
}