use crate::{utils::{DataReadExt, DataWriteExt}, State, uuid::UUID3};

use super::{ReadExactPacket, WriteExactPacket};

#[derive(Debug, Clone)]
pub struct LoginSuccess {
    pub uuid: UUID3,
    pub username: String,
    pub properties: Option<Vec<Property>>,
}

#[derive(Debug, Clone)]
pub struct Property {
    pub name: String,
    pub value: String,
    pub signature: Option<String>,
}

#[async_trait::async_trait]
impl ReadExactPacket for LoginSuccess {
    async fn read_packet(
        mut reader: impl DataReadExt + std::marker::Send, 
        state: &State
    ) -> anyhow::Result<Self> where Self: Sized {
        if state.handshake.is_none() {
            return Err(anyhow::anyhow!("Handshake packet not received"));
        }
        let handshake = state.handshake.as_ref().unwrap();

        let uuid = if handshake.protocol_version >= 735 {
            reader.read_uuid().await.unwrap()
        } else {
            UUID3::try_from(reader.read_string().await?)?
        };
        let username = reader.read_string().await.unwrap();

        let mut properties: Option<Vec<Property>> = None;

        if handshake.protocol_version >= 759 {
            properties = Some(vec![]);

            let n = reader.read_varint().await?;
            for _ in 0..n {
                let name = reader.read_string().await?;
                let value = reader.read_string().await?;

                let signature = if reader.read_bool().await.unwrap() {
                    Some(reader.read_string().await?)
                } else {
                    None
                };

                properties.as_mut().unwrap().push(Property {
                    name, value, signature,
                });
            }
        }

        Ok(Self {
            uuid,
            username,
            properties,
        })
    }
}

#[async_trait::async_trait]
impl WriteExactPacket for LoginSuccess {
    async fn write_packet(
        &self, 
        mut writer: impl DataWriteExt + std::marker::Send, 
        state: &State
    ) -> anyhow::Result<()> {
        if state.handshake.is_none() {
            return Err(anyhow::anyhow!("Handshake packet not received"));
        }
        let handshake = state.handshake.as_ref().unwrap();

        let mut data = vec![];

        if handshake.protocol_version >= 735 {
            data.write_uuid(self.uuid).await?;
        } else {
            data.write_string(&self.uuid.to_string()).await?;
        }

        data.write_string(&self.username).await?;

        if handshake.protocol_version >= 759 {
            if self.properties.is_none() {
                return Err(anyhow::anyhow!("Properties is None, but protocol version >= 759"));
            }
            let properties = self.properties.as_ref().unwrap();

            data.write_varint(properties.len() as i32).await?;

            for property in properties {
                data.write_string(&property.name).await?;
                data.write_string(&property.value).await?;

                if let Some(signature) = &property.signature {
                    data.write_bool(true).await?;
                    data.write_string(signature).await?;
                } else {
                    data.write_bool(false).await?;
                }
            }
        }
        writer.write_varint(data.len() as i32 + 1).await?;
        writer.write_u8(0x02).await?; // TODO: some of protocol versions user another packet id
        writer.write_all(&data).await?;

        Ok(())
    }
}