use crate::crc::crc16;

/// Magic bytes that open every Bloop packet.
pub const MAGIC: &[u8; 2] = b"BL";
pub const VERSION: u8 = 1;

/// Minimum packet size: 16-byte header + 2-byte CRC, no payload.
const MIN_SIZE: usize = 18;

/// Wire format:
///
/// ```text
/// magic:        2 bytes  "BL"
/// version:      1 byte
/// flags:        1 byte
/// sender_id:    4 bytes  big-endian u32
/// packet_id:    4 bytes  big-endian u32
/// symbol_count: 2 bytes  big-endian u16
/// payload_len:  2 bytes  big-endian u16
/// payload:      n bytes
/// crc16:        2 bytes  big-endian u16  (covers all preceding bytes)
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BloopPacket {
    pub version: u8,
    pub flags: u8,
    pub sender_id: u32,
    pub packet_id: u32,
    /// Number of 5-bit symbols encoded in the payload.
    pub symbol_count: u16,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PacketError {
    InvalidMagic,
    UnknownVersion(u8),
    CrcMismatch { expected: u16, actual: u16 },
    BufferTooShort { needed: usize, available: usize },
}

impl std::fmt::Display for PacketError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PacketError::InvalidMagic => write!(f, "invalid magic bytes (expected \"BL\")"),
            PacketError::UnknownVersion(v) => write!(f, "unknown packet version: {}", v),
            PacketError::CrcMismatch { expected, actual } => {
                write!(f, "CRC mismatch: expected {:#06x}, got {:#06x}", expected, actual)
            }
            PacketError::BufferTooShort { needed, available } => {
                write!(f, "buffer too short: need {} bytes, have {}", needed, available)
            }
        }
    }
}

impl std::error::Error for PacketError {}

pub fn encode_packet(packet: &BloopPacket) -> Result<Vec<u8>, PacketError> {
    let payload_len = packet.payload.len();
    let mut buf = Vec::with_capacity(MIN_SIZE + payload_len);

    buf.extend_from_slice(MAGIC);
    buf.push(packet.version);
    buf.push(packet.flags);
    buf.extend_from_slice(&packet.sender_id.to_be_bytes());
    buf.extend_from_slice(&packet.packet_id.to_be_bytes());
    buf.extend_from_slice(&packet.symbol_count.to_be_bytes());
    buf.extend_from_slice(&(payload_len as u16).to_be_bytes());
    buf.extend_from_slice(&packet.payload);

    let checksum = crc16(&buf);
    buf.extend_from_slice(&checksum.to_be_bytes());

    Ok(buf)
}

pub fn decode_packet(bytes: &[u8]) -> Result<BloopPacket, PacketError> {
    if bytes.len() < MIN_SIZE {
        return Err(PacketError::BufferTooShort {
            needed: MIN_SIZE,
            available: bytes.len(),
        });
    }

    if &bytes[0..2] != MAGIC {
        return Err(PacketError::InvalidMagic);
    }

    let version = bytes[2];
    if version != VERSION {
        return Err(PacketError::UnknownVersion(version));
    }

    let flags = bytes[3];
    let sender_id = u32::from_be_bytes(bytes[4..8].try_into().unwrap());
    let packet_id = u32::from_be_bytes(bytes[8..12].try_into().unwrap());
    let symbol_count = u16::from_be_bytes(bytes[12..14].try_into().unwrap());
    let payload_len = u16::from_be_bytes(bytes[14..16].try_into().unwrap()) as usize;

    let needed = MIN_SIZE + payload_len;
    if bytes.len() < needed {
        return Err(PacketError::BufferTooShort {
            needed,
            available: bytes.len(),
        });
    }

    let body_end = 16 + payload_len;
    let declared_crc = u16::from_be_bytes(bytes[body_end..body_end + 2].try_into().unwrap());
    let computed_crc = crc16(&bytes[..body_end]);

    if computed_crc != declared_crc {
        return Err(PacketError::CrcMismatch {
            expected: declared_crc,
            actual: computed_crc,
        });
    }

    Ok(BloopPacket {
        version,
        flags,
        sender_id,
        packet_id,
        symbol_count,
        payload: bytes[16..body_end].to_vec(),
    })
}
