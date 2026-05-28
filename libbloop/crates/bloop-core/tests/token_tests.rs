use bloop_core::error::BloopError;
use bloop_core::symbol::Symbol;
use bloop_core::token::{symbols_to_tokens, tokens_to_symbols, Token};

// --- Normal text produces Symbol tokens ---

#[test]
fn plain_text_becomes_symbol_tokens() {
    let symbols = vec![Symbol::H, Symbol::I];
    let tokens = symbols_to_tokens(&symbols).unwrap();
    assert_eq!(tokens, vec![Token::Symbol(Symbol::H), Token::Symbol(Symbol::I)]);
}

#[test]
fn empty_symbol_stream_produces_no_tokens() {
    let tokens = symbols_to_tokens(&[]).unwrap();
    assert!(tokens.is_empty());
}

// --- Escape token parsing ---

#[test]
fn single_escape_token() {
    let symbols = vec![Symbol::Esc, Symbol::Space, Symbol::A]; // ESC bank=0 index=1
    let tokens = symbols_to_tokens(&symbols).unwrap();
    assert_eq!(tokens, vec![Token::Escape { bank: 0, index: 1 }]);
}

#[test]
fn escape_with_known_bank_and_index() {
    // ESC bank=0 index=5 — canonical punctuation bank, entry 5
    let symbols = vec![Symbol::Esc, Symbol::Space, Symbol::E];
    let tokens = symbols_to_tokens(&symbols).unwrap();
    assert_eq!(tokens, vec![Token::Escape { bank: 0, index: 5 }]);
}

#[test]
fn multiple_escape_tokens() {
    let symbols = vec![
        Symbol::Esc, Symbol::Space, Symbol::A, // ESC 0 1
        Symbol::Esc, Symbol::B,    Symbol::C,  // ESC 2 3
    ];
    let tokens = symbols_to_tokens(&symbols).unwrap();
    assert_eq!(
        tokens,
        vec![
            Token::Escape { bank: 0, index: 1 },
            Token::Escape { bank: 2, index: 3 },
        ]
    );
}

#[test]
fn mixed_symbols_and_escapes() {
    let symbols = vec![
        Symbol::H, Symbol::I,
        Symbol::Esc, Symbol::Space, Symbol::A,
        Symbol::Z,
    ];
    let tokens = symbols_to_tokens(&symbols).unwrap();
    assert_eq!(
        tokens,
        vec![
            Token::Symbol(Symbol::H),
            Token::Symbol(Symbol::I),
            Token::Escape { bank: 0, index: 1 },
            Token::Symbol(Symbol::Z),
        ]
    );
}

#[test]
fn unknown_bank_still_parses() {
    // Bank 15, index 31 — reserved but structurally valid
    let symbols = vec![Symbol::Esc, Symbol::O, Symbol::Esc];
    // O=15, Esc=31
    let tokens = symbols_to_tokens(&symbols).unwrap();
    assert_eq!(tokens, vec![Token::Escape { bank: 15, index: 31 }]);
}

// --- Dangling / incomplete escape errors ---

#[test]
fn dangling_esc_at_end_of_stream() {
    let symbols = vec![Symbol::A, Symbol::Esc];
    let result = symbols_to_tokens(&symbols);
    assert_eq!(result, Err(BloopError::DanglingEscape { position: 1 }));
}

#[test]
fn dangling_esc_only_stream() {
    let symbols = vec![Symbol::Esc];
    let result = symbols_to_tokens(&symbols);
    assert_eq!(result, Err(BloopError::DanglingEscape { position: 0 }));
}

#[test]
fn incomplete_escape_missing_index() {
    let symbols = vec![Symbol::Esc, Symbol::Space]; // has bank, missing index
    let result = symbols_to_tokens(&symbols);
    assert_eq!(result, Err(BloopError::IncompleteEscape { position: 0 }));
}

#[test]
fn incomplete_escape_after_valid_token() {
    let symbols = vec![Symbol::A, Symbol::Esc, Symbol::B]; // A then ESC B (no index)
    let result = symbols_to_tokens(&symbols);
    assert_eq!(result, Err(BloopError::IncompleteEscape { position: 1 }));
}

// --- Round-trip tokens_to_symbols ---

#[test]
fn escape_token_round_trips() {
    let tokens = vec![Token::Escape { bank: 0, index: 5 }];
    let symbols = tokens_to_symbols(&tokens).unwrap();
    assert_eq!(symbols, vec![Symbol::Esc, Symbol::Space, Symbol::E]);
}

#[test]
fn mixed_token_round_trips() {
    let tokens = vec![
        Token::Symbol(Symbol::H),
        Token::Escape { bank: 1, index: 2 },
        Token::Symbol(Symbol::Z),
    ];
    let symbols = tokens_to_symbols(&tokens).unwrap();
    let back = symbols_to_tokens(&symbols).unwrap();
    assert_eq!(back, tokens);
}

// --- tokens_to_symbols rejects bank/index > 31 ---

#[test]
fn tokens_to_symbols_rejects_bank_out_of_range() {
    let tokens = vec![Token::Escape { bank: 32, index: 0 }];
    let result = tokens_to_symbols(&tokens);
    assert_eq!(result, Err(BloopError::InvalidSymbolValue(32)));
}

#[test]
fn tokens_to_symbols_rejects_index_out_of_range() {
    let tokens = vec![Token::Escape { bank: 0, index: 255 }];
    let result = tokens_to_symbols(&tokens);
    assert_eq!(result, Err(BloopError::InvalidSymbolValue(255)));
}
