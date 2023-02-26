use std::net::SocketAddr;

use protocol::{PacketReadExt, error::ProtocolError, PacketWriteExt, State, packets::Packet};
use tokio::{
    net::{tcp::{OwnedReadHalf, OwnedWriteHalf}, TcpStream}, 
    io::AsyncWriteExt
};

use crate::transformers::transform_packet;

pub struct Upstream(pub OwnedReadHalf, pub OwnedWriteHalf, pub SocketAddr);

pub struct ProxyConnection {
    pub remote_addr: SocketAddr,
    
    pub upstream: (OwnedReadHalf, OwnedWriteHalf),
    pub downstream: (OwnedReadHalf, OwnedWriteHalf),
}

enum PipeType {
    C2S, S2C,
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
        
        let upstream = pipe(&mut self.upstream.0, &mut self.downstream.1, PipeType::C2S);
        let downstream = pipe(&mut self.downstream.0, &mut self.upstream.1, PipeType::S2C);

        let _ = tokio::join!(upstream, downstream);
    }
}

async fn update_state(state: &mut State, packet: &Packet) {
    match packet {
        Packet::C2S(packet) => {
            match packet {
                protocol::packets::C2SPacket::Handshake(packet) => {
                    *state = match packet.next_state {
                        protocol::packets::c2s::NextState::Status => State::Status,
                        protocol::packets::c2s::NextState::Login => State::Login,
                    };
                },
                _ => {},
            }
        },
        Packet::S2C(_) => {},
    }
}

async fn pipe(reader: &mut OwnedReadHalf, writer: &mut OwnedWriteHalf, pipe_type: PipeType) -> anyhow::Result<()> {
    let str_type = match pipe_type {
        PipeType::C2S => "C2S",
        PipeType::S2C => "S2C",
    };

    // TODO: Both pipes should have shared state.
    // TODO: Remake this to PipeState or smth like that
    let mut state = State::Handshake;

    loop {
        let packet = match pipe_type {
            PipeType::C2S => {
                reader.read_packet_c2s(state).await
            },
            PipeType::S2C => {
                reader.read_packet_s2c(state).await
            },
        };

        if let Err(e) = packet {
            match e {
                ProtocolError::UnknownPacketId { packet_id: _, data } => {
                    // println!("Unknown packet id ({}): {}", str_type, packet_id);
                    writer.write_all(&data).await?;
                    continue;
                },
                ProtocolError::ReadPacket { source } => {
                    println!("Error reading packet ({}): {}", str_type, source);
                    break;
                },
            }
        }

        let mut packet = packet.unwrap();
        
        update_state(&mut state, &packet).await;
        transform_packet(&mut packet).await;

        writer.write_packet(&packet).await?;
    }

    Ok(())
}