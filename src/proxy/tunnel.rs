use std::{sync::Arc, net::SocketAddr};

use protocol::{State, DirectionEnum, PacketReadExt, error::ProtocolError, packets::{Packet, C2SPacket, c2s::NextState, S2CPacket}, GameStateEnum, PacketWriteExt};
use tokio::{sync::Mutex, net::tcp::{OwnedWriteHalf, OwnedReadHalf}, io::AsyncWriteExt};

pub struct TunnelPipe {
    upstream_addr: SocketAddr,
    state: State
}

impl TunnelPipe {
    pub fn new(upstream_addr: SocketAddr) -> Self {
        Self {
            upstream_addr,
            state: State::default()
        }
    }

    pub async fn establish_pipes(&mut self, upstream: &mut (OwnedReadHalf, OwnedWriteHalf), downstream: &mut (OwnedReadHalf, OwnedWriteHalf)) {
        let arc = Arc::new(Mutex::new(self));

        let a = pipe(arc.clone(), &mut upstream.0, &mut downstream.1, DirectionEnum::C2S);
        let b = pipe(arc, &mut downstream.0, &mut upstream.1, DirectionEnum::S2C);

        let _ = tokio::join!(a, b);
    }

    pub fn update_state(&mut self, packet: &Packet) {
        // let old_state = self.state.clone();

        match packet {
            Packet::C2S(packet) => {
                match (packet, self.state.state) {
                    (C2SPacket::Handshake(packet), GameStateEnum::Handshake) => {
                        self.state.state = match packet.next_state {
                            NextState::Login => GameStateEnum::Login,
                            NextState::Status => GameStateEnum::Status,
                        };
                        self.state.handshake = Some(packet.clone());
                    },
                    _ => {}
                }
            }
            Packet::S2C(packet) => {
                match (packet, self.state.state) {
                    (S2CPacket::LoginSuccess(_packet), GameStateEnum::Login) => {
                        self.state.state = GameStateEnum::Play;
                    },
                    _ => {}
                }
            },
        }
    }
}

async fn pipe(tunnel: Arc<Mutex<&mut TunnelPipe>>, reader: &mut OwnedReadHalf, writer: &mut OwnedWriteHalf, direction: DirectionEnum) -> anyhow::Result<()> {
    let dir_str: &str = direction.into();
    loop {
        let t = tunnel.lock().await;
        /*if direction == DirectionEnum::S2C && t.state.handshake.is_none() {
            continue;
        }*/
        let state = t.state.clone();
        drop(t);

        let packet = reader.read_packet(state, direction).await;

        if let Err(e) = packet {
            match e {
                ProtocolError::UnknownPacketId { packet_id, data } => {
                    println!("Unknown packet id {} from {} ({} bytes)", packet_id, dir_str, data.len());
                    writer.write_all(&data).await?;
                    continue;
                },
                ProtocolError::ReadPacket { source } => {
                    println!("Error reading packet from {}: {}", dir_str, source);
                    break;
                },
            }
        }

        let packet = packet.unwrap();
        println!("Parsed packet from {}: {:#?}", dir_str, packet);
        tunnel.lock().await.update_state(&packet);
        writer.write_packet(&packet).await?;
    }
    Ok(())
}