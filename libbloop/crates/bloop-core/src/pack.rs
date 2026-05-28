use crate::error::BloopError;
use crate::symbol::Symbol;

/// Pack symbols into a byte buffer using consecutive 5-bit fields, MSB first.
pub fn pack_symbols(symbols: &[Symbol]) -> Vec<u8> {
    let bit_count = symbols.len() * 5;
    let byte_count = (bit_count + 7) / 8;
    let mut bytes = vec![0u8; byte_count];

    for (i, &sym) in symbols.iter().enumerate() {
        let value = sym as u8;
        let bit_pos = i * 5;
        let byte_idx = bit_pos / 8;
        let bit_offset = bit_pos % 8;

        if bit_offset <= 3 {
            // All 5 bits fit within byte_idx.
            bytes[byte_idx] |= value << (3 - bit_offset);
        } else {
            // Symbol spans byte_idx and byte_idx+1.
            let bits_in_second = bit_offset - 3; // 1..=4
            bytes[byte_idx] |= value >> bits_in_second;
            bytes[byte_idx + 1] |= value << (8 - bits_in_second);
        }
    }

    bytes
}

/// Unpack exactly `symbol_count` symbols from a packed byte buffer.
pub fn unpack_symbols(bytes: &[u8], symbol_count: usize) -> Result<Vec<Symbol>, BloopError> {
    if symbol_count == 0 {
        return Ok(Vec::new());
    }

    let needed = (symbol_count * 5 + 7) / 8;
    if bytes.len() < needed {
        return Err(BloopError::InsufficientData {
            needed,
            available: bytes.len(),
        });
    }

    let mut symbols = Vec::with_capacity(symbol_count);

    for i in 0..symbol_count {
        let bit_pos = i * 5;
        let byte_idx = bit_pos / 8;
        let bit_offset = bit_pos % 8;

        let value: u8 = if bit_offset <= 3 {
            (bytes[byte_idx] >> (3 - bit_offset)) & 0b11111
        } else {
            let bits_in_second = bit_offset - 3; // 1..=4
            let bits_in_first = 5 - bits_in_second;
            let high = bytes[byte_idx] & ((1 << bits_in_first) - 1);
            let low = bytes[byte_idx + 1] >> (8 - bits_in_second);
            (high << bits_in_second) | low
        };

        symbols.push(Symbol::try_from(value)?);
    }

    Ok(symbols)
}
