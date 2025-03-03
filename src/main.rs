mod packet;

use std::any::Any;
use std::io::Read;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpSocket, TcpStream};
use crate::packet::{ClientBoundStatusPingResponsePacket, ClientBoundStatusResponsePacket, EServerStatusPacket, Packet, PacketFactory, PacketManager, PacketStatus, ServerBoundHandshakePacket, ServerBoundStatusPingPacket};

type PacketType = i32;

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
        let val = (self.data[self.read_offset] as u16) << 8 | self.data[self.read_offset + 1] as u16;
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
        let val = (self.data[self.read_offset] as i64) << 56 |
            (self.data[self.read_offset + 1] as i64) << 48 |
            (self.data[self.read_offset + 2] as i64) << 40 |
            (self.data[self.read_offset + 3] as i64) << 32 |
            (self.data[self.read_offset + 4] as i64) << 24 |
            (self.data[self.read_offset + 5] as i64) << 16 |
            (self.data[self.read_offset + 6] as i64) << 8 |
            (self.data[self.read_offset + 7] as i64);
        self.read_offset += 8;
        val
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
        // Print data BEFORE string write
        println!("Data BEFORE string write");
        for b in self.data.bytes().map(|b| b.unwrap()) {
            println!("Byte: {:02X}", b);
        }

        self.write_varint(val.len() as i32);
        println!("Wrote string len {}", val.len());
        // Print buffer data
        for b in self.data.bytes().map(|b| b.unwrap()) {
            println!("Byte: {:02X}", b);
        }
        self.write_buf(val.as_bytes());
        println!("Wrote string {}", val);
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
                break
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

struct ClientConnection<'a> {
    socket: tokio::sync::MutexGuard<'a, TcpStream>,
    state: i32,
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

        println!("Offset: {}", buf.data.len());
        println!("Buffer with len: {:?}", buf.data.iter().map(|b| format!("{:02X}", b)).collect::<Vec<String>>().join(" "));

        buf.write_buf(&content_buf.data);

        // Write final buffer to socket
        let data = buf.read_all();

        {
            self.socket.write_all(&data).await.unwrap();
            self.socket.flush().await.unwrap();
        }

        println!("= Packet data {:?}", data.iter().map(|b| format!("{:02X}", b)).collect::<Vec<String>>().join(" "));
    }
}

pub async fn read_complete<'a>(connection: &mut ClientConnection<'a>, buf: &mut ByteBuf) {
    let packet_type = buf.read_varint();
    println!("--------------------");
    println!("Received packet type: {}", packet_type);

    match PacketStatus::from_id(connection.state) {
        Some(PacketStatus::Handshake) => {
            println!("Handshake");
            let packet: Box<ServerBoundHandshakePacket> = PacketManager::HANDSHAKE.decode(packet_type, buf);
            connection.state = packet.next_state;
            //println!("Server Handshake packet: {:?}", packet);
        }
        Some(PacketStatus::Status) => {
            let t = PacketManager::STATUS.from_id(packet_type).expect(format!("Unknown status packet type: {}", packet_type).as_str());

            match t {
                EServerStatusPacket::ServerBoundStatusRequestPacket => {
                    println!("Server StatusRequestPacket");

                    let packet_to_send: Box<ClientBoundStatusResponsePacket> = Box::new(ClientBoundStatusResponsePacket::default());
                    connection.send_packet(packet_to_send).await;
                    println!("-> Sent response");
                }
                EServerStatusPacket::ServerBoundStatusPingPacket => {
                    let packet: Box<ServerBoundStatusPingPacket> = PacketManager::STATUS.decode(packet_type, buf);
                    println!("Server StatusPingPacket {}", packet.timestamp);

                    let packet_to_send: Box<ClientBoundStatusPingResponsePacket> = Box::new(ClientBoundStatusPingResponsePacket {
                        timestamp: packet.timestamp,
                    });
                    connection.send_packet(packet_to_send).await;
                    println!("-> Sent response");
                }
            }
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
            };

            //println!("Accepted connection from: {}", addr);

            let mut buf = ByteBuf::new();
            let mut packet_len: i32 = 0;

            // Read data from the socket
            loop {
                let mut read_buf = vec![0; 4096];
                let read_size = connection.socket.read(&mut read_buf).await.unwrap();

                if read_size == 0 {
                    println!("Connection closed: {}", connection.socket.peer_addr().unwrap());
                    break
                }

                buf.write_buf(&read_buf);

                if packet_len == 0 && buf.len() > 0 {
                    packet_len = buf.read_varint();
                    println!("Packet len: {}", packet_len);
                }

                // Wait for buf len == packet_len
                if packet_len != 0 && buf.len() >= packet_len as usize {
                    println!("Packet read complete of len {}", packet_len);
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
    use crate::ByteBuf;

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

}
