use bloop_core::codec::{decode_text, encode_text};
use bloop_core::error::BloopError;
use bloop_core::pack::{pack_symbols, unpack_symbols};
use bloop_core::symbol::Symbol;

// --- Round-trip text encode/decode ---

#[test]
fn round_trip_empty() {
    let enc = encode_text("").unwrap();
    assert_eq!(enc.symbol_count, 0);
    assert_eq!(enc.bytes.len(), 0);
    let dec = decode_text(&enc.bytes, enc.symbol_count).unwrap();
    assert_eq!(dec, "");
}

#[test]
fn round_trip_hello() {
    let enc = encode_text("hello").unwrap();
    assert_eq!(enc.symbol_count, 5);
    let dec = decode_text(&enc.bytes, enc.symbol_count).unwrap();
    assert_eq!(dec, "hello");
}

#[test]
fn round_trip_hello_world() {
    let enc = encode_text("hello world").unwrap();
    assert_eq!(enc.symbol_count, 11);
    let dec = decode_text(&enc.bytes, enc.symbol_count).unwrap();
    assert_eq!(dec, "hello world");
}

#[test]
fn round_trip_pipe_expression() {
    let input = "i saw |a dog|";
    let enc = encode_text(input).unwrap();
    let dec = decode_text(&enc.bytes, enc.symbol_count).unwrap();
    assert_eq!(dec, input);
}

#[test]
fn round_trip_question() {
    let input = "what?";
    let enc = encode_text(input).unwrap();
    let dec = decode_text(&enc.bytes, enc.symbol_count).unwrap();
    assert_eq!(dec, input);
}

#[test]
fn round_trip_multiline() {
    let input = "weather looks wrong\nbring tea";
    let enc = encode_text(input).unwrap();
    let dec = decode_text(&enc.bytes, enc.symbol_count).unwrap();
    assert_eq!(dec, input);
}

// --- Known sizes ---

#[test]
fn sixty_four_symbols_is_forty_bytes() {
    // 64 * 5 = 320 bits = 40 bytes exactly
    let input = "a".repeat(64);
    let enc = encode_text(&input).unwrap();
    assert_eq!(enc.symbol_count, 64);
    assert_eq!(enc.bytes.len(), 40);
}

#[test]
fn one_twenty_eight_symbols_is_eighty_bytes() {
    // 128 * 5 = 640 bits = 80 bytes exactly
    let input = "a".repeat(128);
    let enc = encode_text(&input).unwrap();
    assert_eq!(enc.symbol_count, 128);
    assert_eq!(enc.bytes.len(), 80);
}

// --- Byte count formula ---

#[test]
fn one_symbol_is_one_byte() {
    // 5 bits → 1 byte (3 bits padding)
    let syms = vec![Symbol::A];
    assert_eq!(pack_symbols(&syms).len(), 1);
}

#[test]
fn eight_symbols_is_five_bytes() {
    // 8 * 5 = 40 bits = 5 bytes exactly
    let syms = vec![Symbol::A; 8];
    assert_eq!(pack_symbols(&syms).len(), 5);
}

// --- Unpack error on insufficient data ---

#[test]
fn unpack_insufficient_data_returns_error() {
    let bytes = vec![0u8; 1];
    let result = unpack_symbols(&bytes, 3); // needs 2 bytes
    assert_eq!(
        result,
        Err(BloopError::InsufficientData {
            needed: 2,
            available: 1
        })
    );
}

// --- Symbol-level pack/unpack round trips ---

#[test]
fn all_symbol_values_round_trip() {
    // Pack one of every valid symbol value (0-30; skip Esc=31 since it has no char)
    let symbols: Vec<Symbol> = (0u8..=30)
        .map(|v| Symbol::try_from(v).unwrap())
        .collect();
    let bytes = pack_symbols(&symbols);
    let unpacked = unpack_symbols(&bytes, symbols.len()).unwrap();
    assert_eq!(unpacked, symbols);
}

#[test]
fn esc_symbol_round_trips_through_pack() {
    // Esc is a valid 5-bit value and should survive pack/unpack
    let symbols = vec![Symbol::Esc];
    let bytes = pack_symbols(&symbols);
    let unpacked = unpack_symbols(&bytes, 1).unwrap();
    assert_eq!(unpacked, vec![Symbol::Esc]);
}
