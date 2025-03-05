use crate::handshake::ServerHandshakePacketFactory;
use crate::login::ServerLoginPacketFactory;
use crate::status::ServerStatusPacketFactory;
use crate::{ByteBuf, PacketType};
use std::any::Any;
use crate::configuration::ServerConfigurationPacketFactory;
// Types

pub enum PacketStatus {
    Handshake = 0,
    Status = 1,
    Login = 2,
    Configuration = 3,
    Play = 4,
}

impl PacketStatus {
    pub fn from_id(id: i32) -> Option<PacketStatus> {
        match id {
            0 => Some(PacketStatus::Handshake),
            1 => Some(PacketStatus::Status),
            2 => Some(PacketStatus::Login),
            3 => Some(PacketStatus::Configuration),
            4 => Some(PacketStatus::Play),
            _ => None,
        }
    }
}

pub trait PacketFactory<E> {
    fn decode<T: Packet>(&self, id: i32, buf: &mut ByteBuf) -> Box<T>;
    fn from_id(&self, id: i32) -> Option<E>;
}

pub struct PacketManager {}


impl PacketManager {
    pub const HANDSHAKE: ServerHandshakePacketFactory = ServerHandshakePacketFactory {};
    pub const STATUS: ServerStatusPacketFactory = ServerStatusPacketFactory {};
    pub const LOGIN: ServerLoginPacketFactory = ServerLoginPacketFactory {};
    pub const CONFIGURATION: ServerConfigurationPacketFactory = ServerConfigurationPacketFactory {};

    pub fn decode<T: Packet>(status: PacketStatus, id: i32, buf: &mut ByteBuf) -> Box<T> {
        match status {
            PacketStatus::Handshake => Self::HANDSHAKE.decode(id, buf),
            PacketStatus::Status => Self::STATUS.decode(id, buf),
            PacketStatus::Login => Self::LOGIN.decode(id, buf),
            PacketStatus::Configuration => Self::CONFIGURATION.decode(id, buf),
            _ => panic!("Unknown packet status"),
        }
    }
}

pub trait Packet: Any + Send + 'static {
    fn id(&self) -> PacketType;
    fn encode(&self, buf: &mut ByteBuf);
    fn decode(&mut self, buf: &mut ByteBuf);
}

// Keepalive 0x27

pub struct ClientBoundPlayKeepAlivePacket {
    pub id: i64,
}

impl Default for ClientBoundPlayKeepAlivePacket {
    fn default() -> Self {
        Self { id: 0 }
    }
}

impl Packet for ClientBoundPlayKeepAlivePacket {
    fn id(&self) -> PacketType {
        0x27
    }

    fn encode(&self, buf: &mut ByteBuf) {
        buf.write_i64(self.id);
    }

    fn decode(&mut self, buf: &mut ByteBuf) {
        self.id = buf.read_i64();
    }
}
