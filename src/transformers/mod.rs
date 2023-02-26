use protocol::packets::Packet;

pub async fn transform_packet(packet: &mut Packet) {
    match packet {
        Packet::C2S(packet) => {
            match packet {
                protocol::packets::C2SPacket::Handshake(packet) => {
                    // TODO: IP forwarding
                },
            }
        },
        Packet::S2C(packet) => {

        },
    }
}