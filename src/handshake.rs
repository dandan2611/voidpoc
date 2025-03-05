use crate::packet::{Packet, PacketFactory};
use crate::{ByteBuf, PacketType};
use std::any::Any;

// HANDSHAKE //

pub enum EServerHandshakePacket {
    ServerBoundHandshakePacket = 0x00,
}
pub struct ServerHandshakePacketFactory {}

impl PacketFactory<EServerHandshakePacket> for ServerHandshakePacketFactory {
    fn decode<T: Packet + 'static>(&self, id: i32, buf: &mut ByteBuf) -> Box<T> {
        let packet: Box<dyn Any> = match id {
            0x00 => Box::new(ServerBoundHandshakePacket::default()) as Box<dyn Any>,
            _ => panic!("Unknown packet id {}", id),
        };
        let mut packet: Box<T> = packet.downcast::<T>().expect("Failed to downcast Packet");

        packet.decode(buf);
        packet
    }

    fn from_id(&self, id: i32) -> Option<EServerHandshakePacket> {
        match id {
            0x00 => Some(EServerHandshakePacket::ServerBoundHandshakePacket),
            _ => None,
        }
    }
}

// PACKETS //

// SERVERBOUND //

// Handshake

#[derive(Debug, Default)]
pub struct ServerBoundHandshakePacket {
    pub protocol_version: i32,
    pub server_address: String,
    pub server_port: u16,
    pub next_state: i32,
}

impl Packet for ServerBoundHandshakePacket {
    fn id(&self) -> PacketType {
        0x00
    }

    fn encode(&self, buf: &mut ByteBuf) {
        buf.write_varint(self.protocol_version);
        buf.write_string(&self.server_address);
        buf.write_u16(self.server_port);
        buf.write_varint(self.next_state);
    }

    fn decode(&mut self, buf: &mut ByteBuf) {
        self.protocol_version = buf.read_varint();
        self.server_address = buf.read_string();
        self.server_port = buf.read_u16();
        self.next_state = buf.read_varint();
    }
}

// CLIENTBOUND //