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
        Arc::new(Self { inner: ipfs, rt })
    }

    pub fn peer_id(&self) -> String {
        self.rt.block_on(async {
            match self.inner.identity(None).await {
                Ok(peer) => peer.peer_id.to_string(),
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