use std::net::SocketAddr;

use tokio::net::TcpListener;

use crate::config::Configuration;

use super::connection::{ProxyConnection, Upstream};

pub struct ProxyServer {
    config: Configuration
}

impl ProxyServer {
    pub fn new(config: Configuration) -> Self {
        Self {
            config
        }
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        let listener = TcpListener::bind(&self.config.bind_address).await?;

        loop {
            let (socket, addr) = listener.accept().await?;

            let (r, w) = socket.into_split();
            let upstream = Upstream(r, w, addr);

            // TODO: choose default downstream server
            let destination: SocketAddr = self.config.downstreams[0].address.clone().parse().unwrap();
            tokio::spawn(async move {
                let mut connection = ProxyConnection::init(upstream, destination).await;
                connection.establish().await;
            });
        }
    }
}