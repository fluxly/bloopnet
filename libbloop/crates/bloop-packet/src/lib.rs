pub mod crc;
pub mod packet;

pub use crc::crc16;
pub use packet::{decode_packet, encode_packet, BloopPacket, PacketError, MAGIC, VERSION};
