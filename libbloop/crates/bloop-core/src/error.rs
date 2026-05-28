use crate::symbol::Symbol;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BloopError {
    UnsupportedCharacter(char),
    NoCharRepresentation(Symbol),
    InvalidSymbolValue(u8),
    InsufficientData { needed: usize, available: usize },
    DanglingEscape { position: usize },
    IncompleteEscape { position: usize },
}

impl std::fmt::Display for BloopError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BloopError::UnsupportedCharacter(c) => write!(f, "unsupported character: {:?}", c),
            BloopError::NoCharRepresentation(s) => {
                write!(f, "symbol has no char representation: {:?}", s)
            }
            BloopError::InvalidSymbolValue(v) => write!(f, "invalid symbol value: {}", v),
            BloopError::InsufficientData { needed, available } => {
                write!(f, "insufficient data: need {} bytes, have {}", needed, available)
            }
            BloopError::DanglingEscape { position } => {
                write!(f, "dangling escape at position {}", position)
            }
            BloopError::IncompleteEscape { position } => {
                write!(f, "incomplete escape at position {} (bank present, index missing)", position)
            }
        }
    }
}

impl std::error::Error for BloopError {}
