use crate::packet::{Packet, PacketFactory};
use crate::{ByteBuf, PacketType, Property};
use std::any::Any;
use uuid::Uuid;

// LOGIN //

#[derive(Eq, PartialEq)]
pub enum EServerLoginPacket {
    ServerBoundLoginStartPacket = 0x00,
    ServerBoundLoginLoginAcknowledgedPacket = 0x03,
}

pub struct ServerLoginPacketFactory {}

impl PacketFactory<EServerLoginPacket> for ServerLoginPacketFactory {
    fn decode<T: Packet + 'static>(&self, id: i32, buf: &mut ByteBuf) -> Box<T> {
        let packet: Box<dyn Any> = match id {
            0x00 => Box::new(ServerBoundLoginStartPacket::default()) as Box<dyn Any>,
            0x03 => Box::new(ServerBoundLoginLoginAcknowledgedPacket::default()) as Box<dyn Any>,
            _ => panic!("Unknown packet id"),
        };
        let mut packet: Box<T> = packet.downcast::<T>().expect("Failed to downcast Packet");

        packet.decode(buf);
        packet
    }

    fn from_id(&self, id: i32) -> Option<EServerLoginPacket> {
        match id {
            0x00 => Some(EServerLoginPacket::ServerBoundLoginStartPacket),
            0x03 => Some(EServerLoginPacket::ServerBoundLoginLoginAcknowledgedPacket),
            _ => None,
        }
    }
}

// PACKETS //

// SERVERBOUND //

// Login Start

pub struct ServerBoundLoginStartPacket {
    pub name: String,
    pub uuid: Uuid,
}

impl Default for ServerBoundLoginStartPacket {
    fn default() -> Self {
        Self {
            name: String::new(),
            uuid: Uuid::new_v4(),
        }
    }
}

impl Packet for ServerBoundLoginStartPacket {
    fn id(&self) -> PacketType {
        0x00
    }

    fn encode(&self, buf: &mut ByteBuf) {
        buf.write_string(&self.name);
        buf.write_uuid(&self.uuid);
    }

    fn decode(&mut self, buf: &mut ByteBuf) {
        self.name = buf.read_string();
        self.uuid = buf.read_uuid();
    }
}

// Login Acknowledged

pub struct ServerBoundLoginLoginAcknowledgedPacket {}

impl Default for ServerBoundLoginLoginAcknowledgedPacket {
    fn default() -> Self {
        Self {}
    }
}

impl Packet for ServerBoundLoginLoginAcknowledgedPacket {
    fn id(&self) -> PacketType {
        0x03
    }

    fn encode(&self, buf: &mut ByteBuf) {
    }

    fn decode(&mut self, buf: &mut ByteBuf) {
    }
}

// CLIENTBOUND //

// Login Success

pub struct ClientBoundLoginLoginSuccessPacket {
    pub uuid: Uuid,
    pub username: String,
    pub properties: Vec<Property>,
}

impl Default for ClientBoundLoginLoginSuccessPacket {
    fn default() -> Self {
        Self {
            uuid: Uuid::new_v4(),
            username: String::new(),
            properties: vec![
                Property {
                    name: "textures".to_string(),
                    value: "ewogICJ0aW1lc3RhbXAiIDogMTc0MTA5NzkwNjQ2OCwKICAicHJvZmlsZUlkIiA6ICI3ZmQyZmQyY2I2ZDc0ZGRmYjY0MjZjMzI5Mjk2YWRmOCIsCiAgInByb2ZpbGVOYW1lIiA6ICJkYW5kYW4yNjExIiwKICAidGV4dHVyZXMiIDogewogICAgIlNLSU4iIDogewogICAgICAidXJsIiA6ICJodHRwOi8vdGV4dHVyZXMubWluZWNyYWZ0Lm5ldC90ZXh0dXJlLzc3YzQ2MzAyYWU2MmRhOTI0MDVmMjRmZGJjN2FmZGFhOTc3NzRiMGRkODg5MjBkODk3MjNiYTlmMDhiZWI5MDkiCiAgICB9CiAgfQp9".to_string(),
                    signature: None,
                }
            ],
        }
    }
}

impl Packet for ClientBoundLoginLoginSuccessPacket {
    fn id(&self) -> PacketType {
        0x02
    }

    fn encode(&self, buf: &mut ByteBuf) {
        buf.write_uuid(&self.uuid);
        buf.write_string(&self.username);
        buf.write_properties(&self.properties);
    }

    fn decode(&mut self, buf: &mut ByteBuf) {
        self.uuid = buf.read_uuid();
        self.username = buf.read_string();
        self.properties = buf.read_properties();
    }
}