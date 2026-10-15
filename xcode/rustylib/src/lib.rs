uniffi::setup_scaffolding!();
 
#[uniffi::export]
fn rust_hello() -> String {
    "Hello from Rust!".to_string()
}

#[uniffi::export]
pub fn rust_add(a: u32, b: u32) -> u32 {
    a + b
}

use rust_ipfs::builder::DefaultIpfsBuilder as IpfsBuilder;
use rust_ipfs::Ipfs;

#[uniffi::export]
pub fn start_ipfs_node() -> String {
    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(err) => return format!("failed to create tokio runtime: {err}"),
    };

    rt.block_on(async {
        let ipfs: Ipfs = match IpfsBuilder::new()
            .with_default()
            .enable_tcp()
            .enable_dns()
            .set_default_listener()
            .start()
            .await
        {
            Ok(ipfs) => ipfs,
            Err(err) => return format!("failed to start ipfs node: {err}"),
        };

        match ipfs.identity(None).await {
            Ok(peer) => peer.peer_id.to_string(),
            Err(err) => format!("failed to get identity: {err}"),
        }
    })
}

