use crate::utils::{DataReadExt, DataWriteExt};

pub mod c2s;
pub mod s2c;

#[derive(Debug, Clone)]
pub enum C2SPacket {
    Handshake(c2s::Handshake),
}

#[derive(Debug, Clone)]
pub enum S2CPacket {
    // Handshake(s2c::Handshake),
}

#[derive(Debug, Clone)]
pub enum Packet {
    C2S(C2SPacket),
    S2C(S2CPacket),
}

#[async_trait::async_trait]
pub trait ReadExactPacket {
    async fn read_packet(mut reader: impl DataReadExt + std::marker::Send) -> anyhow::Result<Self> where Self: Sized;
}

#[async_trait::async_trait]
pub trait WriteExactPacket {
    async fn write_packet(&self, mut writer: impl DataWriteExt + std::marker::Send) -> anyhow::Result<()>;
}
