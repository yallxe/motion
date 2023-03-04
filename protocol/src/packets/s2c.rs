use crate::{utils::{DataReadExt, DataWriteExt}, State};

use super::{ReadExactPacket, WriteExactPacket};

#[derive(Debug, Clone)]
pub struct LoginSuccess {
    pub uuid: String,
    pub username: String,
}

#[async_trait::async_trait]
impl ReadExactPacket for LoginSuccess {
    async fn read_packet(mut reader: impl DataReadExt + std::marker::Send, _state: State) -> anyhow::Result<Self> where Self: Sized {
        let uuid = reader.read_string().await.unwrap();
        let username = reader.read_string().await.unwrap();

        // TODO: Handle new 'properties' field on newer versions

        Ok(Self {
            uuid,
            username,
        })
    }
}

#[async_trait::async_trait]
impl WriteExactPacket for LoginSuccess {
    async fn write_packet(&self, mut writer: impl DataWriteExt + std::marker::Send) -> anyhow::Result<()> {
        let mut data = vec![];

        data.write_string(&self.uuid).await?;
        data.write_string(&self.username).await?;

        writer.write_varint(data.len() as i32 + 1).await?;
        writer.write_u8(0x02).await?;
        writer.write_all(&data).await?;

        Ok(())
    }
}