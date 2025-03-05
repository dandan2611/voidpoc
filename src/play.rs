use crate::packet::{Packet, PacketFactory};
use crate::{ByteBuf, PacketType};
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::fs::ReadDir;
use bit_set::BitSet;
use crab_nbt::nbt;
use simdnbt::owned::{BaseNbt, Nbt, NbtCompound, NbtTag};
// STATUS //

#[derive(Eq, PartialEq)]
pub enum EServerPlayPacket {
}

pub struct ServerConfigurationPacketFactory {}

impl PacketFactory<EServerPlayPacket> for ServerConfigurationPacketFactory {
    fn decode<T: Packet + 'static>(&self, id: i32, buf: &mut ByteBuf) -> Box<T> {
        let packet: Box<dyn Any> = match id {
            _ => panic!("Unknown packet id"),
        };
        let mut packet: Box<T> = packet.downcast::<T>().expect("Failed to downcast Packet");

        packet.decode(buf);
        packet
    }

    fn from_id(&self, id: i32) -> Option<EServerPlayPacket> {
        match id {
            _ => None,
        }
    }
}

// PACKETS //

// SERVERBOUND //

// Acknowledge Finish Configuration 0x03

// CLIENTBOUND //

// Clientbound Login

pub struct ClientBoundPlayLoginPacket {
    pub entity_id: i32,
    pub hardcore: bool,
    pub dimension_names: Vec<String>,
    pub max_players: i32,
    pub view_distance: i32,
    pub simulation_distance: i32,
    pub reduced_debug_info: bool,
    pub respawn_screen: bool,
    pub limited_crafing: bool,
    pub dimension_type: i32,
    pub dimension_name: String,
    pub hashed_seed: i64,
    pub game_mode: u8,
    pub previous_game_mode: i8,
    pub is_debug: bool,
    pub is_flat: bool,
    pub has_death_location: bool,
    pub death_dimension_name: Option<bool>,
    pub death_location: Option<bool>,
    pub portal_cooldown: i32,
    pub sea_level: i32,
    pub enforces_secure_chat: bool,
}

impl Default for ClientBoundPlayLoginPacket {
    fn default() -> Self {
        ClientBoundPlayLoginPacket {
            entity_id: 1,
            hardcore: false,
            dimension_names: vec!["minecraft:overworld".to_string()],
            max_players: 100,
            view_distance: 10,
            simulation_distance: 10,
            reduced_debug_info: false,
            respawn_screen: true,
            limited_crafing: false,
            dimension_type: 0,
            dimension_name: "minecraft:overworld".to_string(),
            hashed_seed: 0,
            game_mode: 0,
            previous_game_mode: 0,
            is_debug: false,
            is_flat: false,
            has_death_location: false,
            death_dimension_name: None,
            death_location: None,
            portal_cooldown: 10,
            sea_level: 63,
            enforces_secure_chat: false,
        }
    }
}

impl Packet for ClientBoundPlayLoginPacket {
    fn id(&self) -> PacketType {
        0x2C
    }

    fn encode(&self, buf: &mut ByteBuf) {
        buf.write_i32(self.entity_id);
        buf.write_bool(self.hardcore);
        buf.write_varint(0);
        /*for name in &self.dimension_names {
            buf.write_string(name);
        }*/
        buf.write_varint(self.max_players);
        buf.write_varint(self.view_distance);
        buf.write_varint(self.simulation_distance);
        buf.write_bool(self.reduced_debug_info);
        buf.write_bool(self.respawn_screen);
        buf.write_bool(self.limited_crafing);

        buf.write_varint(self.dimension_type);
        buf.write_string(&self.dimension_name);
        buf.write_i64(self.hashed_seed);
        buf.write_i8(0);
        buf.write_i8(self.previous_game_mode);
        buf.write_bool(self.is_debug);
        buf.write_bool(self.is_flat);
        buf.write_i8(0);
        buf.write_varint(self.portal_cooldown);
        buf.write_varint(self.sea_level);
        buf.write_bool(self.enforces_secure_chat);
    }

    fn decode(&mut self, buf: &mut ByteBuf) {
        todo!()
    }
}

// Set Chunk Center 0x58

pub struct ClientBoundPlaySetChunkCenterPacket {
    pub chunk_x: i32,
    pub chunk_z: i32,
}

impl Default for ClientBoundPlaySetChunkCenterPacket {
    fn default() -> Self {
        ClientBoundPlaySetChunkCenterPacket {
            chunk_x: 0,
            chunk_z: 0,
        }
    }
}

impl Packet for ClientBoundPlaySetChunkCenterPacket {
    fn id(&self) -> PacketType {
        0x58
    }

    fn encode(&self, buf: &mut ByteBuf) {
        buf.write_varint(self.chunk_x);
        buf.write_varint(self.chunk_z);
    }

    fn decode(&mut self, buf: &mut ByteBuf) {
        todo!()
    }
}

// Chunk Data

pub struct ChunkData {
    pub heightmaps: Nbt,
    pub data: Vec<i8>,
}

pub struct LightData {
    pub sky_light_mask: Vec<i8>,
    pub block_light_mask: Vec<i8>,
    pub empty_sky_light_mask: Vec<i8>,
    pub empty_block_light_mask: Vec<i8>,
}

pub struct ClientBoundPlayChunkDataPacket {
    pub chunk_x: i32,
    pub chunk_z: i32,
}

fn encode_empty_bitset(buf: &mut ByteBuf) {
    let mut bs = BitSet::new();
    let mut longs = Vec::new();
    for x in bs.iter() {
        longs.push(x);
    }
    buf.write_varint(longs.len() as i32);
    for long in longs {
        buf.write_i64(long as i64);
    }
}

impl Packet for ClientBoundPlayChunkDataPacket {
    fn id(&self) -> PacketType {
        0x28
    }

    fn encode(&self, buf: &mut ByteBuf) {
        println!("Dump before buffer encode");
        buf.dump();

        buf.write_i32(self.chunk_x);
        buf.write_i32(self.chunk_z);
        println!("Dump after chunk x and z");
        buf.dump();
        // Chunk data
        /*let nbt = nbt!("", {});
        {
            let bytes = nbt.write();
            let mut bb = ByteBuf::new();
            bb.write_buf(&bytes);
            let _ = bb.pop_front_i8();
            buf.write_buf(&bb.data);
        }*/
        //buf.write_u8(10);
        //buf.write_u32(1);
        //buf.write_u8('a' as u8);
        //buf.write_u16(0);
        buf.write_u8(0);
        println!("Dump after nbt");
        buf.dump();
        buf.write_varint(0);
        buf.write_varint(0);
        //
        println!("Dump after data");
        buf.dump();
        //
        // Light data
        encode_empty_bitset(buf);
        encode_empty_bitset(buf);
        encode_empty_bitset(buf);
        encode_empty_bitset(buf);
        buf.write_varint(0);
        buf.write_varint(0);
    }

    fn decode(&mut self, buf: &mut ByteBuf) {
        todo!()
    }
}

// Game Event 0x23

pub struct ClientBoundPlayGameEventPacket {
    pub event_id: i8,
    pub value: f32,
}

impl Packet for ClientBoundPlayGameEventPacket {
    fn id(&self) -> PacketType {
        0x23
    }

    fn encode(&self, buf: &mut ByteBuf) {
        buf.write_i8(self.event_id);
        buf.write_f32(self.value);
    }

    fn decode(&mut self, buf: &mut ByteBuf) {
        todo!()
    }
}

// Player Position 0x42

pub struct ClientBoundPlayPlayerPosition {
    pub teleport_id: i32,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub vel_x: f64,
    pub vel_y: f64,
    pub vel_z: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub flags: i16,
}

impl Default for ClientBoundPlayPlayerPosition {
    fn default() -> Self {
        ClientBoundPlayPlayerPosition {
            teleport_id: 0,
            x: 8.0,
            y: 25.0,
            z: 8.0,
            vel_x: 0.0,
            vel_y: 0.0,
            vel_z: 0.0,
            yaw: 0.0,
            pitch: 0.0,
            flags: 0,
        }
    }
}

impl Packet for ClientBoundPlayPlayerPosition {
    fn id(&self) -> PacketType {
        0x42
    }

    fn encode(&self, buf: &mut ByteBuf) {
        buf.write_varint(self.teleport_id);
        buf.write_f64(self.x);
        buf.write_f64(self.y);
        buf.write_f64(self.z);
        buf.write_f64(self.vel_x);
        buf.write_f64(self.vel_y);
        buf.write_f64(self.vel_z);
        buf.write_f32(self.yaw);
        buf.write_f32(self.pitch);
        buf.write_i16(self.flags);
    }

    fn decode(&mut self, buf: &mut ByteBuf) {
        todo!()
    }
}
