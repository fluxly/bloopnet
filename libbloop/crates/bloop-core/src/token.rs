use crate::error::BloopError;
use crate::symbol::Symbol;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Symbol(Symbol),
    Escape { bank: u8, index: u8 },
}

/// Parse a flat symbol stream into tokens, consuming ESC BANK INDEX triples.
pub fn symbols_to_tokens(symbols: &[Symbol]) -> Result<Vec<Token>, BloopError> {
    let mut tokens = Vec::new();
    let mut i = 0;

    while i < symbols.len() {
        if symbols[i] == Symbol::Esc {
            if i + 1 >= symbols.len() {
                return Err(BloopError::DanglingEscape { position: i });
            }
            if i + 2 >= symbols.len() {
                return Err(BloopError::IncompleteEscape { position: i });
            }
            let bank = symbols[i + 1] as u8;
            let index = symbols[i + 2] as u8;
            tokens.push(Token::Escape { bank, index });
            i += 3;
        } else {
            tokens.push(Token::Symbol(symbols[i]));
            i += 1;
        }
    }

    Ok(tokens)
}

/// Flatten tokens back to a symbol stream. Returns an error if bank or index > 31.
pub fn tokens_to_symbols(tokens: &[Token]) -> Result<Vec<Symbol>, BloopError> {
    let mut symbols = Vec::new();

    for token in tokens {
        match token {
            Token::Symbol(s) => symbols.push(*s),
            Token::Escape { bank, index } => {
                symbols.push(Symbol::Esc);
                symbols.push(Symbol::try_from(*bank)?);
                symbols.push(Symbol::try_from(*index)?);
            }
        }
    }

    Ok(symbols)
}
