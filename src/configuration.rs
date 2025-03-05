use crate::packet::{Packet, PacketFactory};
use crate::{ByteBuf, PacketType};
use serde::{Deserialize, Serialize};
use std::any::Any;
use simdnbt::owned::{BaseNbt, Nbt, NbtCompound, NbtTag};
// STATUS //

#[derive(Eq, PartialEq)]
pub enum EServerConfigurationPacket {
    ServerBoundConfigurationAcknowledgeFinishConfiguration = 0x03,
}

pub struct ServerConfigurationPacketFactory {}

impl PacketFactory<EServerConfigurationPacket> for ServerConfigurationPacketFactory {
    fn decode<T: Packet + 'static>(&self, id: i32, buf: &mut ByteBuf) -> Box<T> {
        let packet: Box<dyn Any> = match id {
            0x03 => Box::new(ServerBoundConfigurationAcknowledgeFinishConfiguration::default()) as Box<dyn Any>,
            _ => panic!("Unknown packet id"),
        };
        let mut packet: Box<T> = packet.downcast::<T>().expect("Failed to downcast Packet");

        packet.decode(buf);
        packet
    }

    fn from_id(&self, id: i32) -> Option<EServerConfigurationPacket> {
        match id {
            0x03 => Some(EServerConfigurationPacket::ServerBoundConfigurationAcknowledgeFinishConfiguration),
            _ => None,
        }
    }
}

// PACKETS //

// SERVERBOUND //

// Acknowledge Finish Configuration 0x03

pub struct ServerBoundConfigurationAcknowledgeFinishConfiguration {}

impl Default for ServerBoundConfigurationAcknowledgeFinishConfiguration {
    fn default() -> Self {
        ServerBoundConfigurationAcknowledgeFinishConfiguration {}
    }
}

impl Packet for ServerBoundConfigurationAcknowledgeFinishConfiguration {
    fn id(&self) -> PacketType {
        0x03
    }

    fn encode(&self, buf: &mut ByteBuf) {
    }

    fn decode(&mut self, buf: &mut ByteBuf) {
    }
}

// CLIENTBOUND //

// Clientbound Known Packs

struct Pack {
    pub namespace: String,
    pub id: String,
    pub version: String,
}

pub struct ClientBoundConfigurationClientBoundKnownPacks {
    pub known_packs: Vec<Pack>,
}

impl Default for ClientBoundConfigurationClientBoundKnownPacks {
    fn default() -> Self {
        ClientBoundConfigurationClientBoundKnownPacks {
            known_packs: vec![
                Pack {
                    namespace: "minecraft".to_string(),
                    id: "core".to_string(),
                    version: "1.21.4".to_string(),
                }
            ]
        }
    }
}

impl Packet for ClientBoundConfigurationClientBoundKnownPacks {
    fn id(&self) -> PacketType {
        0x0E
    }

    fn encode(&self, buf: &mut ByteBuf) {
        buf.write_varint(self.known_packs.len() as i32);
        for pack in &self.known_packs {
            buf.write_string(&pack.namespace);
            buf.write_string(&pack.id);
            buf.write_string(&pack.version);
        }
    }

    fn decode(&mut self, buf: &mut ByteBuf) {
        let count = buf.read_varint();
        for _ in 0..count {
            self.known_packs.push(Pack {
                namespace: buf.read_string(),
                id: buf.read_string(),
                version: buf.read_string(),
            });
        }
    }
}

// Registry Data 0x07

pub struct RegistryEntry {
    pub identifier: String,
    pub data: Option<Nbt>,
}

pub struct ClientBoundConfigurationRegistryDataPacket {
    pub identifier: String,
    pub entries: Vec<RegistryEntry>,
}

impl Packet for ClientBoundConfigurationRegistryDataPacket {
    fn id(&self) -> PacketType {
        0x07
    }

    fn encode(&self, buf: &mut ByteBuf) {
        buf.write_string(&self.identifier);
        buf.write_varint(self.entries.len() as i32);
        for entry in &self.entries {
            buf.write_string(&entry.identifier);
            buf.write_i8(0);
            /*match &entry.data {
                Some(data) => {
                    buf.write_i8(1);
                    buf.write_nbt(data);
                }
                None => {
                    buf.write_i8(0);
                }
            }*/
        }
    }

    fn decode(&mut self, buf: &mut ByteBuf) {
        todo!()
    }
}

// Finish Configuration 0x03

pub struct ClientBoundConfigurationFinishConfigurationPacket {}

impl Default for ClientBoundConfigurationFinishConfigurationPacket {
    fn default() -> Self {
        ClientBoundConfigurationFinishConfigurationPacket {}
    }
}

impl Packet for ClientBoundConfigurationFinishConfigurationPacket {
    fn id(&self) -> PacketType {
        0x03
    }

    fn encode(&self, buf: &mut ByteBuf) {
    }

    fn decode(&mut self, buf: &mut ByteBuf) {
    }
}
