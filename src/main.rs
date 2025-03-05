mod configuration;
mod handshake;
mod login;
mod packet;
mod status;
mod play;

use crate::configuration::{ClientBoundConfigurationClientBoundKnownPacks, ClientBoundConfigurationFinishConfigurationPacket, ClientBoundConfigurationRegistryDataPacket, EServerConfigurationPacket, RegistryEntry};
use crate::handshake::ServerBoundHandshakePacket;
use crate::login::{
    ClientBoundLoginLoginSuccessPacket, EServerLoginPacket, ServerBoundLoginStartPacket,
};
use crate::packet::{ClientBoundPlayKeepAlivePacket, Packet, PacketFactory, PacketManager, PacketStatus};
use crate::status::{
    ClientBoundStatusPingResponsePacket, ClientBoundStatusResponsePacket, EServerStatusPacket,
    ServerBoundStatusPingPacket,
};
use simdnbt::owned::{BaseNbt, Nbt, NbtCompound, NbtTag};
use std::io::Read;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use uuid::Uuid;
use crate::play::{ClientBoundPlayChunkDataPacket, ClientBoundPlayGameEventPacket, ClientBoundPlayLoginPacket, ClientBoundPlayPlayerPosition, ClientBoundPlaySetChunkCenterPacket};

type PacketType = i32;

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
struct Property {
    name: String,
    value: String,
    signature: Option<String>,
}

struct ByteBuf {
    data: Vec<u8>,
    read_offset: usize,
}

impl ByteBuf {
    const SEGMENT_BITS: i32 = 0x7F;
    const CONTINUE_BIT: i32 = 0x80;

    pub fn new() -> ByteBuf {
        ByteBuf {
            data: Vec::new(),
            read_offset: 0,
        }
    }

    pub fn dump(&self) {
        for i in 0..self.data.len() {
            print!("{:02X} ", self.data[i]);
        }
        println!();
    }

    pub fn pop_front_i8(&mut self) -> i8 {
        let val = self.data[0] as i8;
        self.data = self.data[1..].to_vec();
        val
    }

    pub fn write_u8(&mut self, val: u8) -> &Self {
        self.data.push(val);
        self
    }

    pub fn read_u8(&mut self) -> u8 {
        let val = self.data[self.read_offset];
        self.read_offset += 1;
        val
    }

    pub fn write_u16(&mut self, val: u16) -> &Self {
        self.write_u8((val >> 8) as u8);
        self.write_u8((val & 0xFF) as u8);
        self
    }

    pub fn read_u16(&mut self) -> u16 {
        let val =
            (self.data[self.read_offset] as u16) << 8 | self.data[self.read_offset + 1] as u16;
        self.read_offset += 2;
        val
    }

    pub fn write_u32(&mut self, val: u32) -> &Self {
        self.write_u8((val >> 24) as u8);
        self.write_u8((val >> 16) as u8);
        self.write_u8((val >> 8) as u8);
        self.write_u8((val) as u8);
        self
    }

    pub fn read_u32(&mut self) -> u32 {
        let val = (self.data[self.read_offset] as u32) << 24
            | (self.data[self.read_offset + 1] as u32) << 16
            | (self.data[self.read_offset + 2] as u32) << 8
            | (self.data[self.read_offset + 3] as u32);
        self.read_offset += 4;
        val
    }

    pub fn write_u64(&mut self, val: u64) -> &Self {
        self.write_u8((val >> 56) as u8);
        self.write_u8((val >> 48) as u8);
        self.write_u8((val >> 40) as u8);
        self.write_u8((val >> 32) as u8);
        self.write_u8((val >> 24) as u8);
        self.write_u8((val >> 16) as u8);
        self.write_u8((val >> 8) as u8);
        self
    }

    pub fn read_u64(&mut self) -> u64 {
        let val = (self.data[self.read_offset] as u64) << 56
            | (self.data[self.read_offset + 1] as u64) << 48
            | (self.data[self.read_offset + 2] as u64) << 40
            | (self.data[self.read_offset + 3] as u64) << 32
            | (self.data[self.read_offset + 4] as u64) << 24
            | (self.data[self.read_offset + 5] as u64) << 16
            | (self.data[self.read_offset + 6] as u64) << 8
            | (self.data[self.read_offset + 7] as u64);
        self.read_offset += 8;
        val
    }

    pub fn write_i8(&mut self, val: i8) -> &Self {
        self.write_u8(val as u8);
        self
    }

    pub fn read_i8(&mut self) -> i8 {
        let val = self.data[self.read_offset] as i8;
        self.read_offset += 1;
        val
    }

    pub fn write_i16(&mut self, val: i16) -> &Self {
        self.write_u8((val >> 8) as u8);
        self.write_u8((val & 0xFF) as u8);
        self
    }

    pub fn write_i32(&mut self, val: i32) -> &Self {
        self.write_u8((val >> 24) as u8);
        self.write_u8((val >> 16) as u8);
        self.write_u8((val >> 8) as u8);
        self.write_u8((val) as u8);
        self
    }

    pub fn write_i64(&mut self, val: i64) -> &Self {
        self.write_u8((val >> 56) as u8);
        self.write_u8((val >> 48) as u8);
        self.write_u8((val >> 40) as u8);
        self.write_u8((val >> 32) as u8);
        self.write_u8((val >> 24) as u8);
        self.write_u8((val >> 16) as u8);
        self.write_u8((val >> 8) as u8);
        self.write_u8(val as u8);
        self
    }

    pub fn read_i64(&mut self) -> i64 {
        let val = (self.data[self.read_offset] as i64) << 56
            | (self.data[self.read_offset + 1] as i64) << 48
            | (self.data[self.read_offset + 2] as i64) << 40
            | (self.data[self.read_offset + 3] as i64) << 32
            | (self.data[self.read_offset + 4] as i64) << 24
            | (self.data[self.read_offset + 5] as i64) << 16
            | (self.data[self.read_offset + 6] as i64) << 8
            | (self.data[self.read_offset + 7] as i64);
        self.read_offset += 8;
        val
    }

    pub fn write_f32(&mut self, val: f32) -> &Self {
        self.write_u32(val.to_bits());
        self
    }

    pub fn read_f32(&mut self) -> f32 {
        f32::from_bits(self.read_u32())
    }

    pub fn write_f64(&mut self, val: f64) -> &Self {
        self.write_u64(val.to_bits());
        self
    }

    pub fn read_f64(&mut self) -> f64 {
        f64::from_bits(self.read_u64())
    }

    pub fn write_varint(&mut self, mut val: i32) -> &Self {
        loop {
            if (val & !ByteBuf::SEGMENT_BITS) == 0 {
                self.write_u8(val as u8);
                return self;
            }
            self.write_u8(((val & ByteBuf::SEGMENT_BITS) | ByteBuf::CONTINUE_BIT) as u8);
            val >>= 7;
        }
    }

    pub fn write_string(&mut self, val: &str) -> &Self {
        self.write_varint(val.len() as i32);
        self.write_buf(val.as_bytes());
        self
    }

    pub fn read_string(&mut self) -> String {
        let len = self.read_varint();
        let mut str = String::new();
        for _ in 0..len {
            str.push(self.read_u8() as char);
        }
        str
    }

    pub fn read_varint(&mut self) -> i32 {
        let mut value = 0;
        let mut position = 0;
        let mut current_byte;

        loop {
            current_byte = self.read_i8();
            value |= (current_byte as i32 & ByteBuf::SEGMENT_BITS) << position;

            if (current_byte as i32 & ByteBuf::CONTINUE_BIT) == 0 {
                break;
            }
            position += 7;
            if position >= 32 {
                panic!("HANNNN");
            }
        }
        value
    }

    pub fn write_buf(&mut self, buf: &[u8]) -> &Self {
        self.data.extend_from_slice(&buf);
        self
    }

    pub fn read_buf(&mut self) -> Vec<u8> {
        let buf = self.data[self.read_offset..].to_vec();
        self.read_offset = self.data.len();
        buf
    }

    pub fn write_uuid(&mut self, uuid: &Uuid) -> &Self {
        // Encode the most significant 64 bits first, then the least significant 64 bits
        self.write_i64(uuid.as_u128() as i64);
        self.write_i64((uuid.as_u128() >> 64) as i64);
        self
    }

    pub fn read_uuid(&mut self) -> Uuid {
        let most_significant = self.read_i64() as u64;
        let least_significant = self.read_i64() as u64;

        Uuid::from_u128((most_significant as u128) | ((least_significant as u128) << 64))
    }

    pub fn write_bool(&mut self, val: bool) -> &Self {
        self.write_u8(if val { 1 } else { 0 });
        self
    }

    pub fn read_bool(&mut self) -> bool {
        self.read_u8() == 1
    }

    pub fn write_property(&mut self, prop: &Property) -> &Self {
        self.write_string(&prop.name);
        self.write_string(&prop.value);
        self.write_bool(prop.signature.is_some());
        if let Some(signature) = &prop.signature {
            self.write_string(signature);
        }
        self
    }

    pub fn read_property(&mut self) -> Property {
        let name = self.read_string();
        let value = self.read_string();
        let signature = if self.read_bool() {
            Some(self.read_string())
        } else {
            None
        };
        Property {
            name,
            value,
            signature,
        }
    }

    pub fn write_properties(&mut self, props: &[Property]) -> &Self {
        self.write_varint(props.len() as i32);
        for prop in props {
            self.write_property(prop);
        }
        self
    }

    pub fn read_properties(&mut self) -> Vec<Property> {
        let len = self.read_varint();
        let mut props = Vec::with_capacity(len as usize);
        for _ in 0..len {
            props.push(self.read_property());
        }
        props
    }

    pub fn write_nbt(&mut self, nbt: &Nbt) -> &Self {
        let mut buf = Vec::new();
        nbt.write(&mut buf);
        self.write_buf(&buf);
        self
    }

    pub fn cursor_at(&mut self, pos: usize) -> std::io::Cursor<&[u8]> {
        std::io::Cursor::new(&self.data[pos..])
    }

    pub fn read_nbt(&mut self) -> Nbt {
        let mut cursor = self.cursor_at(self.read_offset);
        let nbt = simdnbt::owned::read(&mut cursor).expect("Failed to read nbt from cursor");
        self.read_offset = cursor.position() as usize;
        nbt
    }

    pub fn write_nbt_compound(&mut self, nbt: &NbtCompound) -> &Self {
        let mut buf = Vec::new();
        nbt.write(&mut buf);
        self.write_buf(&buf);
        self
    }

    pub fn read_nbt_compound(&mut self) -> NbtCompound {
        let mut cursor = self.cursor_at(self.read_offset);
        let nbt = simdnbt::owned::read_compound(&mut cursor).expect("Failed to read nbt compound from cursor");
        self.read_offset = cursor.position() as usize;
        nbt
    }

    pub fn clear(&mut self) -> &mut Self {
        self.data.clear();
        self.read_offset = 0;
        self
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn remaining_len(&self) -> usize {
        self.data.len() - self.read_offset
    }

    pub fn reset_read_offset(&mut self) {
        self.read_offset = 0;
    }

    pub fn read_all(&mut self) -> Vec<u8> {
        let buf = self.data.clone();
        self.clear();
        buf
    }
}

struct ClientIdentity {
    uuid: Uuid,
    username: String,
}

struct ClientConnection<'a> {
    socket: tokio::sync::MutexGuard<'a, TcpStream>,
    state: i32,
    identity: Option<ClientIdentity>,
}

impl<'a> ClientConnection<'a> {
    pub async fn send_packet(&mut self, packet: Box<dyn Packet>) {
        // Encode packet
        let mut content_buf = ByteBuf::new();
        content_buf.write_varint(packet.id());
        packet.encode(&mut content_buf);

        // Len buffer
        let mut p_id_buf = ByteBuf::new();
        let len = content_buf.len();
        p_id_buf.write_varint(len as i32);

        // Final buffer
        let mut buf = ByteBuf::new();
        buf.write_varint(content_buf.len() as i32);

        buf.write_buf(&content_buf.data);

        // Write final buffer to socket
        let data = buf.read_all();

        {
            self.socket.write_all(&data).await.unwrap();
            self.socket.flush().await.unwrap();
        }
    }
}

fn generate_registry_nbt(name: String) -> RegistryEntry {
    RegistryEntry {
        identifier: "minecraft:".to_string() + &name,
        data: Some(Nbt::Some(BaseNbt::new(
            "",
            NbtCompound::from_values(vec![
                ("message_id".into(), NbtTag::String("arrow".into())),
                ("scaling".into(), NbtTag::String("never".into())),
                ("exhaustion".into(), NbtTag::Float(0.0)),
            ])
        )))
    }
}

pub async fn read_complete<'a>(connection: &mut ClientConnection<'a>, buf: &mut ByteBuf) {
    let packet_type = buf.read_varint();
    println!("--------------------");
    println!("Received packet type: {}", packet_type);

    match PacketStatus::from_id(connection.state) {
        Some(PacketStatus::Handshake) => {
            let packet: Box<ServerBoundHandshakePacket> =
                PacketManager::HANDSHAKE.decode(packet_type, buf);
            connection.state = packet.next_state;
        }
        Some(PacketStatus::Status) => {
            let t = PacketManager::STATUS
                .from_id(packet_type)
                .expect(format!("Unknown status packet type: {}", packet_type).as_str());

            match t {
                EServerStatusPacket::ServerBoundStatusRequestPacket => {
                    let packet_to_send: Box<ClientBoundStatusResponsePacket> =
                        Box::new(ClientBoundStatusResponsePacket::default());
                    connection.send_packet(packet_to_send).await;
                }
                EServerStatusPacket::ServerBoundStatusPingPacket => {
                    let packet: Box<ServerBoundStatusPingPacket> =
                        PacketManager::STATUS.decode(packet_type, buf);
                    let packet_to_send: Box<ClientBoundStatusPingResponsePacket> =
                        Box::new(ClientBoundStatusPingResponsePacket {
                            timestamp: packet.timestamp,
                        });
                    connection.send_packet(packet_to_send).await;
                }
            }
        }
        Some(PacketStatus::Login) => {
            let t = PacketManager::LOGIN
                .from_id(packet_type)
                .expect(format!("Unknown status packet type: {}", packet_type).as_str());

            match t {
                EServerLoginPacket::ServerBoundLoginStartPacket => {
                    let packet: Box<ServerBoundLoginStartPacket> =
                        PacketManager::LOGIN.decode(packet_type, buf);
                    println!("Login start from {} ({})", packet.uuid, packet.name);

                    connection.identity = Some(ClientIdentity {
                        uuid: packet.uuid,
                        username: packet.name.clone(),
                    });

                    let packet = Box::new(ClientBoundLoginLoginSuccessPacket::default());
                    connection.send_packet(packet).await;
                }
                EServerLoginPacket::ServerBoundLoginLoginAcknowledgedPacket => {
                    connection.state = 3;

                    // Print the identity
                    if let Some(identity) = &connection.identity {
                        println!(
                            "Login acknowledged from: {} ({})",
                            identity.username, identity.uuid
                        );
                    }

                    // Send packs
                    let p = Box::new(ClientBoundConfigurationClientBoundKnownPacks::default());
                    connection.send_packet(p).await;

                    // Send registries
                    // Biomes
                    let p = Box::new(ClientBoundConfigurationRegistryDataPacket {
                        identifier: "minecraft:worldgen/biome".to_string(),
                        entries: vec![
                            RegistryEntry {
                                identifier: "minecraft:old_growth_pine_taiga".to_string(),
                                data: Some(Nbt::Some(BaseNbt::new(
                                    "",
                                    NbtCompound::from_values(vec![
                                        ("has_precipitation".into(), NbtTag::Byte(0)),
                                        ("temperature".into(), NbtTag::Float(1.0)),
                                        ("downfall".into(), NbtTag::Float(0.0)),
                                        ("effects".into(), NbtTag::Compound(NbtCompound::from_values(vec![
                                            ("fog_color".into(), NbtTag::Int(8364543)),
                                            ("water_color".into(), NbtTag::Int(8364543)),
                                            ("water_fog_color".into(), NbtTag::Int(8364543)),
                                            ("sky_color".into(), NbtTag::Int(8364543)),
                                        ]))),
                                    ])
                                )))
                            },
                            RegistryEntry {
                                identifier: "minecraft:plains".to_string(),
                                data: Some(Nbt::Some(BaseNbt::new(
                                    "",
                                    NbtCompound::from_values(vec![
                                        ("has_precipitation".into(), NbtTag::Byte(0)),
                                        ("temperature".into(), NbtTag::Float(1.0)),
                                        ("downfall".into(), NbtTag::Float(0.0)),
                                        ("effects".into(), NbtTag::Compound(NbtCompound::from_values(vec![
                                            ("fog_color".into(), NbtTag::Int(8364543)),
                                            ("water_color".into(), NbtTag::Int(8364543)),
                                            ("water_fog_color".into(), NbtTag::Int(8364543)),
                                            ("sky_color".into(), NbtTag::Int(8364543)),
                                        ]))),
                                    ])
                                )))
                            }
                        ]
                    });
                    connection.send_packet(p).await;

                    // Wolf variants
                    let p = Box::new(ClientBoundConfigurationRegistryDataPacket {
                        identifier: "minecraft:wolf_variant".to_string(),
                        entries: vec![
                            RegistryEntry {
                                identifier: "minecraft:black".to_string(),
                                data: Some(Nbt::Some(BaseNbt::new(
                                    "",
                                    NbtCompound::from_values(vec![
                                        ("wild_texture".into(), NbtTag::String("minecraft:entity/wolf/wolf_ashen".into())),
                                        ("tame_texture".into(), NbtTag::String("minecraft:entity/wolf/wolf_ashen".into())),
                                        ("angry_texture".into(), NbtTag::String("minecraft:entity/wolf/wolf_ashen".into())),
                                        ("biomes".into(), NbtTag::String("minecraft:old_growth_pine_taiga".into())),
                                    ])
                                )))
                            }
                        ]
                    });
                    connection.send_packet(p).await;

                    // Painting variants
                    let p = Box::new(ClientBoundConfigurationRegistryDataPacket {
                        identifier: "minecraft:painting_variant".to_string(),
                        entries: vec![
                            RegistryEntry {
                                identifier: "minecraft:backyard".to_string(),
                                data: Some(Nbt::Some(BaseNbt::new(
                                    "",
                                    NbtCompound::from_values(vec![
                                        ("asset_id".into(), NbtTag::String("minecraft:backyard".into())),
                                        ("height".into(), NbtTag::Int(1)),
                                        ("width".into(), NbtTag::Int(1)),
                                        ("title".into(), NbtTag::String("{\"color\": \"gray\", \"translate\": \"painting.minecraft.skeleton.title\"}".into())),
                                        ("author".into(), NbtTag::String("{\"color\": \"gray\", \"translate\": \"painting.minecraft.skeleton.title\"}".into())),
                                    ])
                                )))
                            }
                        ]
                    });
                    connection.send_packet(p).await;

                    // Damage type
                    // arrow, bad_respawn_point, cactus, campfire, cramming, dragon_breath, drown, dry_out, ender_pearl, explosion, fall, falling_anvil, falling_block, falling_stalactite, fireball, fireworks, fly_into_wall, freeze, generic, generic_kill, hot_floor, in_fire, in_wall, indirect_magic, lava, lightning_bolt, mace_smash, magic, mob_attack, mob_attack_no_aggro, mob_projectile, on_fire, out_of_world, outside_border, player_attack, player_explosion, sonic_boom, spit, stalagmite, starve, sting, sweet_berry_bush, thorns, thrown, trident, unattributed_fireball, wind_charge, wither, wither_skull
                    let mut pvec = Vec::new();
                    // For each damage type, generate a registry entry
                    for damage_type in vec![
                        "arrow", "bad_respawn_point", "cactus", "campfire", "cramming", "dragon_breath", "drown", "dry_out", "ender_pearl", "explosion", "fall", "falling_anvil", "falling_block", "falling_stalactite", "fireball", "fireworks", "fly_into_wall", "freeze", "generic", "generic_kill", "hot_floor", "in_fire", "in_wall", "indirect_magic", "lava", "lightning_bolt", "mace_smash", "magic", "mob_attack", "mob_attack_no_aggro", "mob_projectile", "on_fire", "out_of_world", "outside_border", "player_attack", "player_explosion", "sonic_boom", "spit", "stalagmite", "starve", "sting", "sweet_berry_bush", "thorns", "thrown", "trident", "unattributed_fireball", "wind_charge", "wither", "wither_skull"
                    ] {
                        pvec.push(generate_registry_nbt(damage_type.to_string()));
                    }
                    let p = Box::new(ClientBoundConfigurationRegistryDataPacket {
                        identifier: "minecraft:damage_type".to_string(),
                        entries: pvec,
                    });
                    connection.send_packet(p).await;

                    // Dimension types
                    let p = Box::new(ClientBoundConfigurationRegistryDataPacket {
                        identifier: "minecraft:dimension_type".to_string(),
                        entries: vec![
                            RegistryEntry {
                                identifier: "minecraft:overworld".to_string(),
                                data: Some(Nbt::Some(BaseNbt::new(
                                    "",
                                    NbtCompound::from_values(vec![
                                        ("fixed_time".into(), NbtTag::Long(12000)),
                                        ("has_skylight".into(), NbtTag::Byte(1)),
                                        ("has_ceiling".into(), NbtTag::Byte(0)),
                                        ("ultrawarm".into(), NbtTag::Byte(0)),
                                        ("natural".into(), NbtTag::Byte(0)),
                                        ("coordinate_scale".into(), NbtTag::Double(1.0)),
                                        ("bed_works".into(), NbtTag::Byte(0)),
                                        ("respawn_anchor_works".into(), NbtTag::Byte(1)),
                                        ("min_y".into(), NbtTag::Int(-100)),
                                        ("height".into(), NbtTag::Int(256)),
                                        ("logical_height".into(), NbtTag::Int(255)),
                                        ("infiniburn".into(), NbtTag::String("#".into())),
                                        ("effects".into(), NbtTag::String("minecraft:overworld".into())),
                                        ("ambient_light".into(), NbtTag::Float(1.0)),
                                        ("piglin_safe".into(), NbtTag::Byte(1)),
                                        ("has_raids".into(), NbtTag::Byte(0)),
                                        ("monster_spawn_light_level".into(), NbtTag::Int(0)),
                                        ("monster_spawn_block_light_limit".into(), NbtTag::Int(0)),
                                    ])
                                )))
                            }
                        ]
                    });
                    connection.send_packet(p).await;

                    // Finish config
                    let p = Box::new(ClientBoundConfigurationFinishConfigurationPacket::default());
                    connection.send_packet(p).await;
                }
                _ => {
                    eprintln!("Unknown login packet type: {}", packet_type);
                }
            }
        }
        Some(PacketStatus::Configuration) => {
            let t = PacketManager::CONFIGURATION
                .from_id(packet_type);

            if (t.is_none()) {
                eprintln!("Unknown configuration packet type: {}", packet_type);
                // Finish config
                return;
            }

            let t = t.unwrap();

            match t {
                EServerConfigurationPacket::ServerBoundConfigurationAcknowledgeFinishConfiguration => {
                    println!("Acknowledge finish configuration");

                    let p = Box::new(ClientBoundPlayLoginPacket::default());
                    connection.send_packet(p).await;

                    connection.state = 4;

                    // Send play packets
                    // Send chunk center
                    let p = Box::new(ClientBoundPlaySetChunkCenterPacket::default());
                    connection.send_packet(p).await;

                    println!("Play set chunk center");

                    // Send game event
                    let p = Box::new(ClientBoundPlayGameEventPacket {
                        event_id: 13,
                        value: 0f32,
                    });
                    connection.send_packet(p).await;

                    println!("Play game event");

                    // Send chunk data
                    let p = Box::new(ClientBoundPlayChunkDataPacket {
                        chunk_x: 0,
                        chunk_z: 0,
                    });
                    //connection.send_packet(p).await;

                    println!("Play chunk data");

                    // Synchronize position
                    let p = Box::new(ClientBoundPlayPlayerPosition::default());
                    //connection.send_packet(p).await;

                    println!("Play player position");
                }
                _ => {
                    eprintln!("Unknown configuration packet type: {}", packet_type);
                }
            }
        }
        Some(PacketStatus::Play) => {
            if packet_type == 11 {
                let p = Box::new(ClientBoundPlayKeepAlivePacket::default());
                connection.send_packet(p).await;
                return;
            }
            if packet_type == 0x1A {
                return;
            }
            eprintln!("Play packet type: {}", packet_type);
        }
        _ => {
            eprintln!("Unknown packet status: {}", packet_type);
        }
    }
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("0.0.0.0:25565").await.unwrap();

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let socket = Arc::new(Mutex::new(socket));

        tokio::spawn(async move {
            let socket = socket.lock().await;

            let mut connection = ClientConnection {
                socket,
                state: 0,
                identity: None,
            };

            //println!("Accepted connection from: {}", addr);

            let mut buf = ByteBuf::new();
            let mut packet_len: i32 = 0;

            // Read data from the socket
            loop {
                let mut read_buf = vec![0; 4096];
                let read_size = connection.socket.read(&mut read_buf).await.unwrap();

                if read_size == 0 {
                    println!(
                        "Connection closed: {}",
                        connection.socket.peer_addr().unwrap()
                    );
                    break;
                }

                buf.write_buf(&read_buf);

                if packet_len == 0 && buf.len() > 0 {
                    packet_len = buf.read_varint();
                    //println!("Packet len: {}", packet_len);
                }

                // Wait for buf len == packet_len
                if packet_len != 0 && buf.len() >= packet_len as usize {
                    //println!("Packet read complete of len {}", packet_len);
                    read_complete(&mut connection, &mut buf).await;
                    buf.clear();
                    packet_len = 0;
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use crate::{ByteBuf, Property};

    #[test]
    pub fn test_buf_write_varint() {
        let mut buf = ByteBuf::new();
        buf.write_varint(156);

        assert_eq!(buf.read_varint(), 156);
        assert_eq!(buf.remaining_len(), 0);
    }

    #[test]
    pub fn test_buf_write_varint_multiple() {
        let mut buf = ByteBuf::new();
        buf.write_varint(2611);
        buf.write_varint(2611);

        assert_eq!(buf.read_varint(), 2611);
        assert_eq!(buf.read_varint(), 2611);
        assert_eq!(buf.remaining_len(), 0);
    }

    #[test]
    pub fn test_buf_write_string() {
        let mut buf = ByteBuf::new();
        buf.write_string("abcde");

        let str = String::from("abcde");
        let buf_str = buf.read_string();

        assert_eq!(str, buf_str);
        assert_eq!(buf.remaining_len(), 0);
    }

    #[test]
    pub fn test_buf_clear() {
        let mut buf = ByteBuf::new();

        buf.write_u8(0u8);
        assert_eq!(buf.len(), 1);
        buf.clear();
        assert_eq!(buf.len(), 0);
    }

    #[test]
    pub fn test_buf_write_uuid() {
        use uuid::Uuid;

        let mut buf = ByteBuf::new();
        let uuid = Uuid::new_v4();
        buf.write_uuid(&uuid);

        let read_uuid = buf.read_uuid();
        assert_eq!(uuid, read_uuid);
        assert_eq!(buf.remaining_len(), 0);
    }

    #[test]
    pub fn test_buf_write_property_no_signature() {
        let mut buf = ByteBuf::new();
        let property = Property {
            name: "name".to_string(),
            value: "value".to_string(),
            signature: None,
        };

        buf.write_property(&property);

        let read_property = buf.read_property();
        assert_eq!(property, read_property);
        assert_eq!(buf.remaining_len(), 0);
    }

    #[test]
    pub fn test_buf_write_property_with_signature() {
        let mut buf = ByteBuf::new();
        let property = Property {
            name: "name".to_string(),
            value: "value".to_string(),
            signature: Some("signature".to_string()),
        };

        buf.write_property(&property);

        let read_property = buf.read_property();
        assert_eq!(property, read_property);
        assert_eq!(buf.remaining_len(), 0);
    }

    #[test]
    pub fn test_buf_write_properties_no_signature() {
        let mut buf = ByteBuf::new();
        let properties = vec![
            Property {
                name: "name".to_string(),
                value: "value".to_string(),
                signature: None,
            },
            Property {
                name: "name2".to_string(),
                value: "value2".to_string(),
                signature: None,
            },
        ];

        buf.write_properties(&properties);

        let read_properties = buf.read_properties();
        assert_eq!(properties, read_properties);
        assert_eq!(buf.remaining_len(), 0);
    }

    #[test]
    pub fn test_buf_write_properties_with_signature() {
        let mut buf = ByteBuf::new();
        let properties = vec![
            Property {
                name: "name".to_string(),
                value: "value".to_string(),
                signature: Some("signature".to_string()),
            },
            Property {
                name: "name2".to_string(),
                value: "value2".to_string(),
                signature: Some("signature2".to_string()),
            },
        ];

        buf.write_properties(&properties);

        let read_properties = buf.read_properties();
        assert_eq!(properties, read_properties);
        assert_eq!(buf.remaining_len(), 0);
    }

    #[test]
    pub fn test_buf_write_properties_mixed() {
        let mut buf = ByteBuf::new();
        let properties = vec![
            Property {
                name: "name".to_string(),
                value: "value".to_string(),
                signature: None,
            },
            Property {
                name: "name2".to_string(),
                value: "value2".to_string(),
                signature: Some("signature2".to_string()),
            },
        ];

        buf.write_properties(&properties);

        let read_properties = buf.read_properties();
        assert_eq!(properties, read_properties);
        assert_eq!(buf.remaining_len(), 0);
    }

    #[test]
    pub fn test_buf_write_f32() {
        let mut buf = ByteBuf::new();
        buf.write_f32(3.14);

        assert_eq!(buf.read_f32(), 3.14);
        assert_eq!(buf.remaining_len(), 0);
    }

    #[test]
    pub fn test_buf_write_nbt_compound() {
        use simdnbt::owned::{BaseNbt, NbtCompound, NbtTag};

        let mut buf = ByteBuf::new();
        let nbt = NbtCompound::from_values(vec![
            ("key".into(), NbtTag::String("value".into())),
        ]);
        buf.write_nbt_compound(&nbt);

        let read_nbt = buf.read_nbt_compound();
        assert_eq!(nbt, read_nbt);
        assert_eq!(buf.remaining_len(), 0);
    }

    #[test]
    pub fn test_buf_pop_front_i8() {
        let mut buf = ByteBuf::new();
        buf.write_u8(0x01);
        buf.write_u8(0x02);
        buf.write_u8(0x03);

        assert_eq!(buf.pop_front_i8(), 0x01);
        assert_eq!(buf.pop_front_i8(), 0x02);
        assert_eq!(buf.pop_front_i8(), 0x03);
        assert_eq!(buf.len(), 0);
    }
}
