use anyhow::Result;

pub mod config;
pub mod proxy;

#[tokio::main]
async fn main() -> Result<()> {
    let config = config::Configuration {
        downstreams: vec![
            config::DownstreamConfig {
                address: "127.0.0.1:25500".to_string(),
                name: "default".to_string(),
                default: true
            }
        ],
        bind_address: "0.0.0.0:25565".to_string(),
    };
    
    let proxy = proxy::server::ProxyServer::new(config);
    Ok(proxy.run().await?)
}
