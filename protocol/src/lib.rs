use error::ProtocolError;
use packets::{Packet, ReadExactPacket, WriteExactPacket, C2SPacket, S2CPacket, c2s, s2c};
use utils::{DataReadExt, DataWriteExt};

pub mod utils;
pub mod packets;
pub mod error;
pub mod uuid;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum GameStateEnum {
    #[default] Handshake,
    Status,
    Login,
    Play,
}

#[derive(Debug, Clone, Default)]
pub struct State {
    pub handshake: Option<c2s::Handshake>,
    pub state: GameStateEnum,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectionEnum {
    C2S, S2C,
}

impl From<DirectionEnum> for &str {
    fn from(direction: DirectionEnum) -> Self {
        match direction {
            DirectionEnum::C2S => "C2S",
            DirectionEnum::S2C => "S2C",
        }
    }
}

#[async_trait::async_trait]
pub trait PacketReadExt: DataReadExt + Unpin {
    async fn read_packet(&mut self, state: &State, direction: DirectionEnum) -> Result<Packet, ProtocolError> {
        match direction {
            DirectionEnum::C2S => self.read_packet_c2s(state).await,
            DirectionEnum::S2C => self.read_packet_s2c(state).await,
        }
    }

    async fn read_packet_c2s(&mut self, state: &State) -> Result<Packet, ProtocolError> {
        let (length, mut d1) = self.read_varint_preserve_data().await?;
        let (packet_id, mut d2) = self.read_varint_preserve_data().await?;

        let packet = match (packet_id, state.state) {
            (0x00, GameStateEnum::Handshake) => {
                let handshake = c2s::Handshake::read_packet(self, state).await?;
                Packet::C2S(C2SPacket::Handshake(handshake))
            },
            (0x00, GameStateEnum::Login) => {
                let login_start = c2s::LoginStart::read_packet(self, state).await?;
                Packet::C2S(C2SPacket::LoginStart(login_start))
            },
            _ => {
                let mut data = vec![];
                data.append(&mut d1);
                let t_sub_len = data.len();
                let mut rest = vec![0; length as usize - d2.len()];
                self.read_exact(&mut rest).await.ok();

                data.append(&mut d2);
                data.append(&mut rest.to_vec());

                assert_eq!(data.len() - t_sub_len, length as usize);
                return Err(ProtocolError::UnknownPacketId { packet_id, data: data });
            },
        };

        Ok(packet)
    }

    async fn read_packet_s2c(&mut self, state: &State) -> Result<Packet, ProtocolError> {
        let (length, mut d1) = self.read_varint_preserve_data().await?;
        let (packet_id, mut d2) = self.read_varint_preserve_data().await?;
        let packet = match (packet_id, state.state) {
            (0x02, GameStateEnum::Login) => {
                let login_success = s2c::LoginSuccess::read_packet(self, state).await?;
                Packet::S2C(S2CPacket::LoginSuccess(login_success))
            },
            _ => {
                let mut data = vec![];
                data.append(&mut d1);
                let t_sub_len = data.len();
                let mut rest = vec![0; length as usize - d2.len()];
                self.read_exact(&mut rest).await.ok();

                data.append(&mut d2);
                data.append(&mut rest.to_vec());

                assert_eq!(data.len() - t_sub_len, length as usize);
                return Err(ProtocolError::UnknownPacketId { packet_id, data: data });
            },
        };

        Ok(packet)
    }
}

impl<T: DataReadExt + Unpin> PacketReadExt for T {}

#[async_trait::async_trait]
pub trait PacketWriteExt: DataWriteExt + Unpin {
    async fn write_packet(&mut self, packet: &Packet, state: &State) -> anyhow::Result<()> {
        match packet {
            Packet::C2S(c2s_packet) => {
                match c2s_packet {
                    C2SPacket::Handshake(handshake) => {
                        handshake.write_packet(self, state).await?;
                    },
                    C2SPacket::LoginStart(login_start) => {
                        login_start.write_packet(self, state).await?;
                    }
                }
            },
            Packet::S2C(s2c_packet) => {
                match s2c_packet {
                    S2CPacket::LoginSuccess(login_success) => {
                        login_success.write_packet(self, state).await?;
                    }
                }
            },
        }

        Ok(())
    }
}

impl<T: DataWriteExt + Unpin> PacketWriteExt for T {}