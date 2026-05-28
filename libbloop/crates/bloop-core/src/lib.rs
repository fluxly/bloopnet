pub mod codec;
pub mod error;
pub mod pack;
pub mod symbol;
pub mod token;
pub mod validate;

use error::BloopError;
use symbol::Symbol;

pub fn text_to_symbols(input: &str) -> Result<Vec<Symbol>, BloopError> {
    input.chars().map(Symbol::try_from).collect()
}

pub fn symbols_to_text(symbols: &[Symbol]) -> Result<String, BloopError> {
    symbols.iter().copied().map(char::try_from).collect()
}
