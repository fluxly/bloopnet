use bloop_packet::{decode_packet, encode_packet, BloopPacket, PacketError, VERSION};

fn basic_packet(payload: Vec<u8>, symbol_count: u16) -> BloopPacket {
    BloopPacket {
        version: VERSION,
        flags: 0,
        sender_id: 0xDEADBEEF,
        packet_id: 1,
        symbol_count,
        payload,
    }
}

// --- Round-trips ---

#[test]
fn round_trip_empty_payload() {
    let pkt = basic_packet(vec![], 0);
    let bytes = encode_packet(&pkt).unwrap();
    let decoded = decode_packet(&bytes).unwrap();
    assert_eq!(decoded.payload, vec![]);
    assert_eq!(decoded.symbol_count, 0);
    assert_eq!(decoded.sender_id, 0xDEADBEEF);
    assert_eq!(decoded.packet_id, 1);
    assert_eq!(decoded.flags, 0);
}

#[test]
fn round_trip_preserves_all_fields() {
    let pkt = BloopPacket {
        version: VERSION,
        flags: 0b00000011,
        sender_id: 0x01020304,
        packet_id: 0xABCDEF00,
        symbol_count: 42,
        payload: vec![0xDE, 0xAD, 0xBE, 0xEF],
    };
    let bytes = encode_packet(&pkt).unwrap();
    let decoded = decode_packet(&bytes).unwrap();
    assert_eq!(decoded.version, VERSION);
    assert_eq!(decoded.flags, 0b00000011);
    assert_eq!(decoded.sender_id, 0x01020304);
    assert_eq!(decoded.packet_id, 0xABCDEF00);
    assert_eq!(decoded.symbol_count, 42);
    assert_eq!(decoded.payload, vec![0xDE, 0xAD, 0xBE, 0xEF]);
}

#[test]
fn round_trip_with_real_bloop_payload() {
    // Encode "hello world" via bloop-core and wrap in a packet.
    let enc = bloop_core::codec::encode_text("hello world").unwrap();
    let pkt = basic_packet(enc.bytes.clone(), enc.symbol_count as u16);
    let wire = encode_packet(&pkt).unwrap();
    let decoded = decode_packet(&wire).unwrap();
    assert_eq!(decoded.payload, enc.bytes);
    assert_eq!(decoded.symbol_count, enc.symbol_count as u16);

    // Confirm the payload decodes back to text.
    let text = bloop_core::codec::decode_text(&decoded.payload, decoded.symbol_count as usize).unwrap();
    assert_eq!(text, "hello world");
}

// --- Size verification ---

#[test]
fn empty_packet_is_18_bytes() {
    // 16-byte header + 0 payload + 2-byte CRC = 18
    let bytes = encode_packet(&basic_packet(vec![], 0)).unwrap();
    assert_eq!(bytes.len(), 18);
}

#[test]
fn bloop_64_symbol_packet_is_58_bytes() {
    // 64 symbols × 5 bits = 320 bits = 40 payload bytes
    // 16-byte header + 40-byte payload + 2-byte CRC = 58
    let payload = vec![0u8; 40];
    let bytes = encode_packet(&basic_packet(payload, 64)).unwrap();
    assert_eq!(bytes.len(), 58);
}

#[test]
fn packet_starts_with_magic_bl() {
    let bytes = encode_packet(&basic_packet(vec![1, 2, 3], 1)).unwrap();
    assert_eq!(&bytes[0..2], b"BL");
}

// --- Error cases ---

#[test]
fn buffer_too_short_for_header() {
    let result = decode_packet(&[0u8; 10]);
    assert_eq!(
        result,
        Err(PacketError::BufferTooShort { needed: 18, available: 10 })
    );
}

#[test]
fn buffer_too_short_for_payload() {
    // Build a valid packet then truncate before the CRC.
    let mut bytes = encode_packet(&basic_packet(vec![1, 2, 3, 4, 5], 1)).unwrap();
    bytes.truncate(bytes.len() - 4); // chop payload + CRC
    let result = decode_packet(&bytes);
    assert!(matches!(result, Err(PacketError::BufferTooShort { .. })));
}

#[test]
fn invalid_magic_detected() {
    let mut bytes = encode_packet(&basic_packet(vec![], 0)).unwrap();
    bytes[0] = 0xFF;
    assert_eq!(decode_packet(&bytes), Err(PacketError::InvalidMagic));
}

#[test]
fn unknown_version_detected() {
    let mut bytes = encode_packet(&basic_packet(vec![], 0)).unwrap();
    bytes[2] = 99;
    // CRC will also be wrong, but version check comes first.
    assert_eq!(decode_packet(&bytes), Err(PacketError::UnknownVersion(99)));
}

#[test]
fn crc_mismatch_detected() {
    let mut bytes = encode_packet(&basic_packet(vec![0xAA, 0xBB], 2)).unwrap();
    // Flip a byte in the payload.
    let payload_start = 16;
    bytes[payload_start] ^= 0xFF;
    assert!(matches!(decode_packet(&bytes), Err(PacketError::CrcMismatch { .. })));
}

#[test]
fn crc_mismatch_on_tampered_crc_field() {
    let mut bytes = encode_packet(&basic_packet(vec![1], 1)).unwrap();
    let last = bytes.len() - 1;
    bytes[last] ^= 0xFF;
    assert!(matches!(decode_packet(&bytes), Err(PacketError::CrcMismatch { .. })));
}
