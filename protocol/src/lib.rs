use error::ProtocolError;
use packets::{Packet, ReadExactPacket, WriteExactPacket};
use utils::{DataReadExt, DataWriteExt};

pub mod utils;
pub mod packets;
pub mod error;

#[derive(Debug, Clone, Copy)]
pub enum State {
    Handshake,
    Status,
    Login,
    Play,
}

#[async_trait::async_trait]
pub trait PacketReadExt: DataReadExt + Unpin {
    async fn read_packet_c2s(&mut self, state: State) -> Result<Packet, ProtocolError> {
        let (length, mut d1) = self.read_varint_preserve_data().await?;
        let (packet_id, mut d2) = self.read_varint_preserve_data().await?;

        let packet = match (packet_id, state) {
            (0x00, State::Handshake) => {
                let handshake = packets::c2s::Handshake::read_packet(self).await?;
                Packet::C2S(packets::C2SPacket::Handshake(handshake))
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

    async fn read_packet_s2c(&mut self, _state: State) -> Result<Packet, ProtocolError> {
        let (length, mut d1) = self.read_varint_preserve_data().await?;
        let (packet_id, mut d2) = self.read_varint_preserve_data().await?;
        let packet = match packet_id {
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
    async fn write_packet(&mut self, packet: &Packet) -> anyhow::Result<()> {
        match packet {
            Packet::C2S(c2s_packet) => {
                match c2s_packet {
                    packets::C2SPacket::Handshake(handshake) => {
                        handshake.write_packet(self).await?;
                    },
                }
            },
            Packet::S2C(_s2c_packet) => {
                
            },
        }

        Ok(())
    }
}

impl<T: DataWriteExt + Unpin> PacketWriteExt for T {}