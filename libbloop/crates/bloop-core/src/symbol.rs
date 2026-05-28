use crate::error::BloopError;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Symbol {
    Space = 0,
    A = 1,
    B = 2,
    C = 3,
    D = 4,
    E = 5,
    F = 6,
    G = 7,
    H = 8,
    I = 9,
    J = 10,
    K = 11,
    L = 12,
    M = 13,
    N = 14,
    O = 15,
    P = 16,
    Q = 17,
    R = 18,
    S = 19,
    T = 20,
    U = 21,
    V = 22,
    W = 23,
    X = 24,
    Y = 25,
    Z = 26,
    Dash = 27,
    Question = 28,
    Pipe = 29,
    Return = 30,
    Esc = 31,
}

impl TryFrom<u8> for Symbol {
    type Error = BloopError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Space),
            1 => Ok(Self::A),
            2 => Ok(Self::B),
            3 => Ok(Self::C),
            4 => Ok(Self::D),
            5 => Ok(Self::E),
            6 => Ok(Self::F),
            7 => Ok(Self::G),
            8 => Ok(Self::H),
            9 => Ok(Self::I),
            10 => Ok(Self::J),
            11 => Ok(Self::K),
            12 => Ok(Self::L),
            13 => Ok(Self::M),
            14 => Ok(Self::N),
            15 => Ok(Self::O),
            16 => Ok(Self::P),
            17 => Ok(Self::Q),
            18 => Ok(Self::R),
            19 => Ok(Self::S),
            20 => Ok(Self::T),
            21 => Ok(Self::U),
            22 => Ok(Self::V),
            23 => Ok(Self::W),
            24 => Ok(Self::X),
            25 => Ok(Self::Y),
            26 => Ok(Self::Z),
            27 => Ok(Self::Dash),
            28 => Ok(Self::Question),
            29 => Ok(Self::Pipe),
            30 => Ok(Self::Return),
            31 => Ok(Self::Esc),
            _ => Err(BloopError::InvalidSymbolValue(value)),
        }
    }
}

impl From<Symbol> for u8 {
    fn from(s: Symbol) -> u8 {
        s as u8
    }
}

impl TryFrom<char> for Symbol {
    type Error = BloopError;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            ' ' => Ok(Self::Space),
            'a'..='z' => Symbol::try_from((c as u8) - b'a' + 1),
            '-' => Ok(Self::Dash),
            '?' => Ok(Self::Question),
            '|' => Ok(Self::Pipe),
            '\n' => Ok(Self::Return),
            _ => Err(BloopError::UnsupportedCharacter(c)),
        }
    }
}

impl TryFrom<Symbol> for char {
    type Error = BloopError;

    fn try_from(s: Symbol) -> Result<Self, Self::Error> {
        let v = s as u8;
        match v {
            0 => Ok(' '),
            1..=26 => Ok((b'a' + v - 1) as char),
            27 => Ok('-'),
            28 => Ok('?'),
            29 => Ok('|'),
            30 => Ok('\n'),
            31 => Err(BloopError::NoCharRepresentation(s)),
            _ => unreachable!(),
        }
    }
}
