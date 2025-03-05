use crate::packet::{Packet, PacketFactory};
use crate::{ByteBuf, PacketType};
use serde::{Deserialize, Serialize};
use std::any::Any;

// STATUS //

#[derive(Eq, PartialEq)]
pub enum EServerStatusPacket {
    ServerBoundStatusRequestPacket = 0x00,
    ServerBoundStatusPingPacket = 0x01,
}

pub struct ServerStatusPacketFactory {}

impl PacketFactory<EServerStatusPacket> for ServerStatusPacketFactory {
    fn decode<T: Packet + 'static>(&self, id: i32, buf: &mut ByteBuf) -> Box<T> {
        let packet: Box<dyn Any> = match id {
            0x00 => Box::new(ServerBoundStatusRequestPacket::default()) as Box<dyn Any>,
            0x01 => Box::new(ServerBoundStatusPingPacket::default()) as Box<dyn Any>,
            _ => panic!("Unknown packet id"),
        };
        let mut packet: Box<T> = packet.downcast::<T>().expect("Failed to downcast Packet");

        packet.decode(buf);
        packet
    }

    fn from_id(&self, id: i32) -> Option<EServerStatusPacket> {
        match id {
            0x00 => Some(EServerStatusPacket::ServerBoundStatusRequestPacket),
            0x01 => Some(EServerStatusPacket::ServerBoundStatusPingPacket),
            _ => None,
        }
    }
}

// PACKETS //

// SERVERBOUND //

// Status Request

#[derive(Debug, Default)]
pub struct ServerBoundStatusRequestPacket {}
impl Packet for ServerBoundStatusRequestPacket {
    fn id(&self) -> PacketType {
        0x00
    }

    fn encode(&self, _buf: &mut ByteBuf) {}

    fn decode(&mut self, _buf: &mut ByteBuf) {}
}

// Ping Request

#[derive(Debug, Default)]
pub struct ServerBoundStatusPingPacket {
    pub timestamp: i64,
}

impl Packet for ServerBoundStatusPingPacket {
    fn id(&self) -> PacketType {
        0x01
    }

    fn encode(&self, buf: &mut ByteBuf) {
        buf.write_i64(self.timestamp);
    }

    fn decode(&mut self, buf: &mut ByteBuf) {
        self.timestamp = buf.read_i64();
    }
}

// CLIENTBOUND //

// Status Response 0x00

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerStatusResponseVersion {
    pub name: String,
    pub protocol: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerStatusResponsePlayersSample {
    pub name: String,
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerStatusResponsePlayers {
    pub max: i32,
    pub online: i32,
    pub sample: Vec<ServerStatusResponsePlayersSample>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerStatusResponseDescription {
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerStatusResponse {
    pub version: ServerStatusResponseVersion,
    pub players: ServerStatusResponsePlayers,
    pub description: ServerStatusResponseDescription,
    //pub favicon: String,
    pub enforces_secure_chat: bool,
}

#[derive(Debug)]
pub struct ClientBoundStatusResponsePacket {
    pub response: ServerStatusResponse,
}

impl Default for ClientBoundStatusResponsePacket {
    fn default() -> Self {
        ClientBoundStatusResponsePacket {
            response: ServerStatusResponse {
                version: ServerStatusResponseVersion {
                    name: "1.21.4".to_string(),
                    protocol: 769,
                },
                players: ServerStatusResponsePlayers {
                    max: 100,
                    online: 0,
                    sample: vec![],
                },
                description: ServerStatusResponseDescription {
                    text: "HANNNNNNNNNN".to_string(),
                },
                //favicon: "".to_string(),
                enforces_secure_chat: false,
            }
        }
    }
}

impl Packet for ClientBoundStatusResponsePacket {
    fn id(&self) -> PacketType {
        0x00
    }

    fn encode(&self, buf: &mut ByteBuf) {
        let json = serde_json::to_string(&self.response).unwrap();
        buf.write_string(&json);
    }

    fn decode(&mut self, buf: &mut ByteBuf) {
        let json = buf.read_string();
        self.response = serde_json::from_str(&json).unwrap();
    }
}

// Ping Response 0x01

#[derive(Debug, Default)]
pub struct ClientBoundStatusPingResponsePacket {
    pub timestamp: i64,
}

impl Packet for ClientBoundStatusPingResponsePacket {
    fn id(&self) -> PacketType {
        0x01
    }

    fn encode(&self, buf: &mut ByteBuf) {
        buf.write_i64(self.timestamp);
    }

    fn decode(&mut self, buf: &mut ByteBuf) {
        self.timestamp = buf.read_i64();
    }
}