use tokio::io::AsyncWriteExt;
use crate::{utils::{DataReadExt, DataWriteExt}, State};

use super::{ReadExactPacket, WriteExactPacket};

/// Handshake packet
#[derive(Debug, Clone)]
pub struct Handshake {
    pub protocol_version: i32,
    pub server_address: String,
    pub server_port: u16,
    pub next_state: NextState,
}

#[derive(Debug, Clone)]
pub enum NextState {
    Status,
    Login,
}

impl From<NextState> for i32 {
    fn from(next_state: NextState) -> Self {
        match next_state {
            NextState::Status => 1,
            NextState::Login => 2,
        }
    }
}

#[async_trait::async_trait]
impl ReadExactPacket for Handshake {
    async fn read_packet(mut reader: impl DataReadExt + std::marker::Send, _state: State) -> anyhow::Result<Self> where Self: Sized {
        let protocol_version = reader.read_varint().await.unwrap();
        let server_address = reader.read_string().await.unwrap();
        let server_port = reader.read_u16().await.unwrap();
        let next_state = match reader.read_varint().await.unwrap() {
            1 => NextState::Status,
            2 => NextState::Login,
            _ => {
                return Err(anyhow::anyhow!("Invalid next state"));
            },
        };

        Ok(Self {
            protocol_version,
            server_address,
            server_port,
            next_state,
        })
    }
}

#[async_trait::async_trait]
impl WriteExactPacket for Handshake {
    async fn write_packet(&self, mut writer: impl DataWriteExt + std::marker::Send) -> anyhow::Result<()> {
        let mut data = vec![];

        data.write_varint(self.protocol_version).await?;
        data.write_string(&self.server_address).await?;
        data.write_u16(self.server_port).await?;
        data.write_varint(self.next_state.clone().into()).await?;

        writer.write_varint(data.len() as i32 + 1).await?;
        writer.write_u8(0x00).await?;
        writer.write_all(&data).await?;

        Ok(())
    }
}