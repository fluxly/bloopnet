use crate::error::BloopError;
use crate::pack::{pack_symbols, unpack_symbols};
use crate::{symbols_to_text, text_to_symbols};

pub struct EncodedBloop {
    pub bytes: Vec<u8>,
    pub symbol_count: usize,
}

pub fn encode_text(input: &str) -> Result<EncodedBloop, BloopError> {
    let symbols = text_to_symbols(input)?;
    let symbol_count = symbols.len();
    let bytes = pack_symbols(&symbols);
    Ok(EncodedBloop { bytes, symbol_count })
}

pub fn decode_text(bytes: &[u8], symbol_count: usize) -> Result<String, BloopError> {
    let symbols = unpack_symbols(bytes, symbol_count)?;
    symbols_to_text(&symbols)
}
