use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum ProtocolError {
    #[snafu(display("Unknown packet id: {}", packet_id))]
    UnknownPacketId { packet_id: i32, data: Vec<u8> },

    #[snafu(display("Failed to read packet: {}", source))]
    ReadPacket { source: anyhow::Error },
}

impl From<anyhow::Error> for ProtocolError {
    fn from(source: anyhow::Error) -> Self {
        ProtocolError::ReadPacket { source }
    }
}
