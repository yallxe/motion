use std::{sync::Arc, net::SocketAddr};

use protocol::{State, DirectionEnum, PacketReadExt, error::ProtocolError, packets::{Packet, C2SPacket, c2s::NextState, S2CPacket}, GameStateEnum, PacketWriteExt, uuid::UUID3};
use tokio::{sync::Mutex, net::tcp::{OwnedWriteHalf, OwnedReadHalf}, io::AsyncWriteExt};

pub struct TunnelPipe {
    upstream_addr: SocketAddr,
    state: State,
    tunnel_state: TunnelState,
}

#[derive(Debug, Clone, Default)]
pub struct TunnelState {
    username: Option<String>,
    waiting_login_start: bool,
}

impl TunnelPipe {
    pub fn new(upstream_addr: SocketAddr) -> Self {
        Self {
            upstream_addr,
            state: State::default(),
            tunnel_state: TunnelState::default(),
        }
    }

    pub async fn establish_pipes(&mut self, upstream: &mut (OwnedReadHalf, OwnedWriteHalf), downstream: &mut (OwnedReadHalf, OwnedWriteHalf)) {
        let arc = Arc::new(Mutex::new(self));

        let a = pipe(arc.clone(), &mut upstream.0, &mut downstream.1, DirectionEnum::C2S);
        let b = pipe(arc, &mut downstream.0, &mut upstream.1, DirectionEnum::S2C);

        let _ = tokio::join!(a, b);
    }

    pub fn transform_packet(&self, packet: &mut Packet) -> anyhow::Result<()> {
        match packet {
            Packet::C2S(packet) => {
                match packet {
                    C2SPacket::Handshake(packet) => {
                        let uuid = UUID3::new("OfflinePlayer:".to_string() + &self.tunnel_state.username.as_ref().unwrap());

                        packet.server_address = [
                            packet.server_address.clone(), 
                            self.upstream_addr.ip().to_string(),
                            uuid.to_string(),
                        ].join("\x00");
                    },
                    _ => {}
                }
            },
            _ => {}
        }

        Ok(())
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
                        self.tunnel_state.waiting_login_start = true;
                    },
                    (C2SPacket::LoginStart(packet), GameStateEnum::Login) => {
                        self.tunnel_state.username = Some(packet.username.clone());
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
        if t.state.state == GameStateEnum::Handshake && direction == DirectionEnum::S2C {
            continue;
        }
        let state = t.state.clone();
        drop(t);

        let packet = reader.read_packet(&state, direction).await;

        if let Err(e) = packet {
            match e {
                ProtocolError::UnknownPacketId { packet_id: _, data } => {
                    writer.write_all(&data).await?;
                    continue;
                },
                ProtocolError::ReadPacket { source } => {
                    println!("Error reading packet from {}: {}", dir_str, source);
                    break;
                },
            }
        }

        let mut packet = packet.unwrap();
        if let Packet::C2S(C2SPacket::Handshake(_)) = packet {
            tunnel.lock().await.update_state(&packet);
            continue;
        }

        {
            let mut t = tunnel.lock().await;
            t.transform_packet(&mut packet).unwrap();
            t.update_state(&packet);
            drop(t);
        }

        if let Packet::C2S(C2SPacket::LoginStart(_)) = packet {
            let t = tunnel.lock().await;

            if let Some(handshake) = t.state.handshake.clone() {
                let mut handshake: Packet = Packet::C2S(C2SPacket::Handshake(handshake));
                t.transform_packet(&mut handshake).unwrap();
                writer.write_packet(&handshake, &state).await.unwrap();
            } else {
                break;
            }
        }

        writer.write_packet(&packet, &state).await.unwrap();
    }
    Ok(())
}