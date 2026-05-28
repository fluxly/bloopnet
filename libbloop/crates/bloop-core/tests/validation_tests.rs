use bloop_core::symbol::Symbol;
use bloop_core::validate::{
    size_class, validate_symbols, validate_text, SizeClass, ValidationIssueKind,
};

// --- Valid input ---

#[test]
fn valid_text_is_ok() {
    let report = validate_text("hello world");
    assert!(report.ok);
    assert!(report.issues.is_empty());
}

#[test]
fn valid_text_with_all_base_punctuation_is_ok() {
    let report = validate_text("are you there?\nyes-ish\ni think |maybe|");
    assert!(report.ok, "{:?}", report.issues);
}

// --- Unsupported characters ---

#[test]
fn uppercase_produces_issue() {
    let report = validate_text("Hello");
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    assert!(matches!(
        report.issues[0].kind,
        ValidationIssueKind::UnsupportedCharacter
    ));
    assert_eq!(report.issues[0].position, Some(0));
}

#[test]
fn multiple_unsupported_chars_each_produce_an_issue() {
    let report = validate_text("Hi!");
    let bad: Vec<_> = report
        .issues
        .iter()
        .filter(|i| matches!(i.kind, ValidationIssueKind::UnsupportedCharacter))
        .collect();
    assert_eq!(bad.len(), 2); // 'H' and '!'
}

#[test]
fn period_produces_unsupported_char_issue() {
    let report = validate_text("hello.");
    assert!(!report.ok);
    assert!(matches!(
        report.issues[0].kind,
        ValidationIssueKind::UnsupportedCharacter
    ));
}

// --- Pipe balance ---

#[test]
fn balanced_pipes_are_ok() {
    let report = validate_text("i saw |a dog|");
    assert!(report.ok);
}

#[test]
fn four_pipes_are_ok() {
    let report = validate_text("|a| and |b|");
    assert!(report.ok);
}

#[test]
fn single_pipe_is_unbalanced() {
    let report = validate_text("i saw |a dog");
    assert!(!report.ok);
    assert!(report
        .issues
        .iter()
        .any(|i| matches!(i.kind, ValidationIssueKind::UnbalancedPipe)));
}

#[test]
fn three_pipes_are_unbalanced() {
    let report = validate_text("|a| |b");
    assert!(!report.ok);
    assert!(report
        .issues
        .iter()
        .any(|i| matches!(i.kind, ValidationIssueKind::UnbalancedPipe)));
}

// --- Escape structure (tested via validate_symbols since Esc has no char) ---

#[test]
fn dangling_esc_produces_issue() {
    let symbols = vec![Symbol::A, Symbol::Esc];
    let report = validate_symbols(&symbols);
    assert!(!report.ok);
    assert!(report
        .issues
        .iter()
        .any(|i| matches!(i.kind, ValidationIssueKind::DanglingEscape)));
}

#[test]
fn incomplete_escape_produces_issue() {
    let symbols = vec![Symbol::Esc, Symbol::Space]; // bank present, index missing
    let report = validate_symbols(&symbols);
    assert!(!report.ok);
    assert!(report
        .issues
        .iter()
        .any(|i| matches!(i.kind, ValidationIssueKind::IncompleteEscape)));
}

#[test]
fn well_formed_escape_with_known_bank_is_ok() {
    // ESC bank=0 index=5
    let symbols = vec![Symbol::Esc, Symbol::Space, Symbol::E];
    let report = validate_symbols(&symbols);
    assert!(report.ok, "{:?}", report.issues);
}

// --- Unknown extension bank ---

#[test]
fn bank_zero_through_seven_are_known() {
    for bank in 0u8..=7 {
        let bank_sym = Symbol::try_from(bank).unwrap();
        let symbols = vec![Symbol::Esc, bank_sym, Symbol::Space];
        let report = validate_symbols(&symbols);
        let has_unknown = report
            .issues
            .iter()
            .any(|i| matches!(i.kind, ValidationIssueKind::UnknownExtensionBank));
        assert!(!has_unknown, "bank {} should be known", bank);
    }
}

#[test]
fn bank_eight_is_unknown() {
    let symbols = vec![Symbol::Esc, Symbol::H, Symbol::Space]; // H=8
    let report = validate_symbols(&symbols);
    assert!(!report.ok);
    assert!(report
        .issues
        .iter()
        .any(|i| matches!(i.kind, ValidationIssueKind::UnknownExtensionBank)));
}

#[test]
fn bank_thirty_one_is_unknown() {
    let symbols = vec![Symbol::Esc, Symbol::Esc, Symbol::Space]; // Esc=31
    let report = validate_symbols(&symbols);
    assert!(!report.ok);
    assert!(report
        .issues
        .iter()
        .any(|i| matches!(i.kind, ValidationIssueKind::UnknownExtensionBank)));
}

// --- Message length ---

#[test]
fn message_at_flood_limit_is_ok() {
    let symbols = vec![Symbol::A; 256];
    let report = validate_symbols(&symbols);
    let too_long = report
        .issues
        .iter()
        .any(|i| matches!(i.kind, ValidationIssueKind::MessageTooLong));
    assert!(!too_long);
}

#[test]
fn message_over_flood_limit_produces_issue() {
    let symbols = vec![Symbol::A; 257];
    let report = validate_symbols(&symbols);
    assert!(!report.ok);
    assert!(report
        .issues
        .iter()
        .any(|i| matches!(i.kind, ValidationIssueKind::MessageTooLong)));
}

// --- Size class ---

#[test]
fn size_class_boundaries() {
    assert_eq!(size_class(0), SizeClass::Pulse);
    assert_eq!(size_class(16), SizeClass::Pulse);
    assert_eq!(size_class(17), SizeClass::Bloop);
    assert_eq!(size_class(64), SizeClass::Bloop);
    assert_eq!(size_class(65), SizeClass::LongBloop);
    assert_eq!(size_class(128), SizeClass::LongBloop);
    assert_eq!(size_class(129), SizeClass::Flood);
    assert_eq!(size_class(256), SizeClass::Flood);
    assert_eq!(size_class(257), SizeClass::TooLong);
}

#[test]
fn pulse_and_bloop_are_polite_lora() {
    assert!(SizeClass::Pulse.is_polite_lora());
    assert!(SizeClass::Bloop.is_polite_lora());
    assert!(!SizeClass::LongBloop.is_polite_lora());
    assert!(!SizeClass::Flood.is_polite_lora());
    assert!(!SizeClass::TooLong.is_polite_lora());
}

#[test]
fn size_class_names() {
    assert_eq!(SizeClass::Pulse.name(), "pulse");
    assert_eq!(SizeClass::Bloop.name(), "bloop");
    assert_eq!(SizeClass::LongBloop.name(), "longbloop");
    assert_eq!(SizeClass::Flood.name(), "flood");
    assert_eq!(SizeClass::TooLong.name(), "too long");
}

// --- Multiple issues in one report ---

#[test]
fn multiple_issues_are_all_collected() {
    // Uppercase + unbalanced pipe in one message
    let report = validate_text("Hello |world");
    assert!(!report.ok);
    let kinds: Vec<_> = report.issues.iter().map(|i| &i.kind).collect();
    assert!(kinds
        .iter()
        .any(|k| matches!(k, ValidationIssueKind::UnsupportedCharacter)));
    assert!(kinds
        .iter()
        .any(|k| matches!(k, ValidationIssueKind::UnbalancedPipe)));
}
