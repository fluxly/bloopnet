use bloop_core::error::BloopError;
use bloop_core::symbol::Symbol;
use bloop_core::{symbols_to_text, text_to_symbols};

// --- Symbol value mappings ---

#[test]
fn a_maps_to_1() {
    assert_eq!(u8::from(Symbol::try_from('a').unwrap()), 1);
}

#[test]
fn z_maps_to_26() {
    assert_eq!(u8::from(Symbol::try_from('z').unwrap()), 26);
}

#[test]
fn space_maps_to_0() {
    assert_eq!(u8::from(Symbol::try_from(' ').unwrap()), 0);
}

#[test]
fn dash_maps_to_27() {
    assert_eq!(u8::from(Symbol::try_from('-').unwrap()), 27);
}

#[test]
fn question_maps_to_28() {
    assert_eq!(u8::from(Symbol::try_from('?').unwrap()), 28);
}

#[test]
fn pipe_maps_to_29() {
    assert_eq!(u8::from(Symbol::try_from('|').unwrap()), 29);
}

#[test]
fn newline_maps_to_30() {
    assert_eq!(u8::from(Symbol::try_from('\n').unwrap()), 30);
}

// --- Unsupported character rejection ---

#[test]
fn uppercase_returns_error() {
    assert_eq!(
        Symbol::try_from('A'),
        Err(BloopError::UnsupportedCharacter('A'))
    );
}

#[test]
fn period_returns_error() {
    assert_eq!(
        Symbol::try_from('.'),
        Err(BloopError::UnsupportedCharacter('.'))
    );
}

#[test]
fn exclamation_returns_error() {
    assert_eq!(
        Symbol::try_from('!'),
        Err(BloopError::UnsupportedCharacter('!'))
    );
}

#[test]
fn digit_returns_error() {
    assert_eq!(
        Symbol::try_from('3'),
        Err(BloopError::UnsupportedCharacter('3'))
    );
}

// --- TryFrom<u8> ---

#[test]
fn u8_0_is_space() {
    assert_eq!(Symbol::try_from(0u8).unwrap(), Symbol::Space);
}

#[test]
fn u8_31_is_esc() {
    assert_eq!(Symbol::try_from(31u8).unwrap(), Symbol::Esc);
}

#[test]
fn u8_32_is_invalid() {
    assert_eq!(
        Symbol::try_from(32u8),
        Err(BloopError::InvalidSymbolValue(32))
    );
}

// --- Esc has no char representation ---

#[test]
fn esc_has_no_char() {
    assert_eq!(
        char::try_from(Symbol::Esc),
        Err(BloopError::NoCharRepresentation(Symbol::Esc))
    );
}

// --- Round-trip text ---

#[test]
fn round_trip_hello() {
    let syms = text_to_symbols("hello").unwrap();
    let text = symbols_to_text(&syms).unwrap();
    assert_eq!(text, "hello");
}

#[test]
fn round_trip_hello_world() {
    let syms = text_to_symbols("hello world").unwrap();
    let text = symbols_to_text(&syms).unwrap();
    assert_eq!(text, "hello world");
}

#[test]
fn round_trip_with_pipe() {
    let input = "i saw |a dog|";
    let syms = text_to_symbols(input).unwrap();
    let text = symbols_to_text(&syms).unwrap();
    assert_eq!(text, input);
}

#[test]
fn round_trip_with_question() {
    let input = "what?";
    let syms = text_to_symbols(input).unwrap();
    let text = symbols_to_text(&syms).unwrap();
    assert_eq!(text, input);
}

#[test]
fn round_trip_multiline() {
    let input = "weather looks wrong\nbring tea";
    let syms = text_to_symbols(input).unwrap();
    let text = symbols_to_text(&syms).unwrap();
    assert_eq!(text, input);
}

#[test]
fn round_trip_with_dash() {
    let input = "self-aware";
    let syms = text_to_symbols(input).unwrap();
    let text = symbols_to_text(&syms).unwrap();
    assert_eq!(text, input);
}

// --- Symbol count sanity ---

#[test]
fn hello_world_is_11_symbols() {
    let syms = text_to_symbols("hello world").unwrap();
    assert_eq!(syms.len(), 11);
}

#[test]
fn empty_string_is_zero_symbols() {
    let syms = text_to_symbols("").unwrap();
    assert_eq!(syms.len(), 0);
}
