use std::convert;

use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::uuid::UUID3;

#[async_trait::async_trait]
pub trait DataReadExt: AsyncReadExt + Unpin {
    async fn read_varint_preserve_data(&mut self) -> anyhow::Result<(i32, Vec<u8>)> {
        let mut num_read = 0;
        let mut result = 0;
        let mut data = vec![];

        loop {
            let read = self.read_u8().await?;
            let value = (read & 0x7f) as i32;
            result |= value << (7 * num_read);

            num_read += 1;
            if num_read > 5 {
                return Err(anyhow::anyhow!("VarInt is too big"));
            }

            data.push(read);

            if read & 0x80 == 0 {
                break;
            }
        }

        Ok((result, data))
    }

    async fn read_varint_sized(&mut self) -> anyhow::Result<(i32, usize)> {
        let mut num_read = 0;
        let mut result = 0;

        loop {
            let read = self.read_u8().await?;
            let value = (read & 0x7f) as i32;
            result |= value << (7 * num_read);

            num_read += 1;
            if num_read > 5 {
                return Err(anyhow::anyhow!("VarInt is too big"));
            }

            if read & 0x80 == 0 {
                break;
            }
        }

        Ok((result, num_read))
    }

    async fn read_varint(&mut self) -> anyhow::Result<i32> {
        let (result, _) = self.read_varint_sized().await?;

        Ok(result)
    }

    async fn read_string(&mut self) -> anyhow::Result<String> {
        let length = self.read_varint().await?;
        let mut buf = vec![0; length as usize];
        self.read_exact(&mut buf).await?;

        Ok(String::from_utf8(buf)?)
    }

    async fn read_uuid(&mut self) -> anyhow::Result<UUID3> {
        let mut buf = [0u8; 16];
        self.read_exact(&mut buf).await?;

        Ok(UUID3::from(u128::from_be_bytes(buf)))
    }

    async fn read_bool(&mut self) -> anyhow::Result<bool> {
        Ok(self.read_u8().await? != 0)
    }
}

impl<T: AsyncReadExt + Unpin> DataReadExt for T {}

#[async_trait::async_trait]
pub trait DataWriteExt: AsyncWriteExt + Unpin {
    async fn write_varint(&mut self, mut value: i32) -> anyhow::Result<()> {
        if value == 0 {
            self.write_u8(0).await?;
            return Ok(());
        }
        
        let mut temp = 0;
        while value != 0 {
            temp = (value & 0b01111111) as u8;
            value = (value >> 7) & (i32::max_value() as i32);
            if value != 0 {
                temp |= 0b10000000;
            }
            self.write_u8(temp).await?;
        }
        Ok(())
    }

    async fn write_string(&mut self, value: &str) -> anyhow::Result<()> {
        self.write_varint(value.len() as i32).await?;
        self.write_all(value.as_bytes()).await?;

        Ok(())
    }

    async fn write_uuid(&mut self, value: UUID3) -> anyhow::Result<()> {
        let converted: [u8; 16] = value.into();
        self.write_all(&converted).await?;

        Ok(())
    }

    async fn write_bool(&mut self, value: bool) -> anyhow::Result<()> {
        self.write_u8(value as u8).await?;

        Ok(())
    }
}

impl<T: AsyncWriteExt + Unpin> DataWriteExt for T {}