use std::{net::SocketAddr, sync::Arc};

use protocol::{PacketReadExt, error::ProtocolError, PacketWriteExt, State, packets::Packet, DirectionEnum};
use tokio::{
    net::{tcp::{OwnedReadHalf, OwnedWriteHalf}, TcpStream}, 
    io::AsyncWriteExt, sync::Mutex
};

use crate::transformers::transform_packet;

use super::tunnel::TunnelPipe;

pub struct Upstream(pub OwnedReadHalf, pub OwnedWriteHalf, pub SocketAddr);

pub struct ProxyConnection {
    pub remote_addr: SocketAddr,
    
    pub upstream: (OwnedReadHalf, OwnedWriteHalf),
    pub downstream: (OwnedReadHalf, OwnedWriteHalf),
}

impl ProxyConnection {
    /// Initialize a new proxy connection struct 
    /// with creating a TCP connection to the downstream server.
    pub async fn init(upstream: Upstream, destination: SocketAddr) -> Self {
        let downstream = TcpStream::connect(destination).await.unwrap();

        Self {
            remote_addr: upstream.2,
            
            upstream: (upstream.0, upstream.1),
            downstream: downstream.into_split(),
        }
    }

    /// Establish proxy connection (create a tunnel basically).
    pub async fn establish(&mut self) {
        let mut tunnel = TunnelPipe::new(self.remote_addr);
        tunnel.establish_pipes(&mut self.upstream, &mut self.downstream).await;
    }
}
