use crate::symbol::Symbol;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationIssueKind {
    UnsupportedCharacter,
    DanglingEscape,
    IncompleteEscape,
    UnbalancedPipe,
    UnknownExtensionBank,
    MessageTooLong,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationIssue {
    pub kind: ValidationIssueKind,
    pub position: Option<usize>,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct ValidationReport {
    pub ok: bool,
    pub issues: Vec<ValidationIssue>,
}

impl ValidationReport {
    fn new() -> Self {
        ValidationReport { ok: true, issues: Vec::new() }
    }

    fn add(&mut self, kind: ValidationIssueKind, position: Option<usize>, message: String) {
        self.ok = false;
        self.issues.push(ValidationIssue { kind, position, message });
    }
}

/// Size classes for Bloop messages. Positions are in symbols.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SizeClass {
    Blip,     // ≤ 16 symbols  / 10 bytes
    Bloop,    // ≤ 64 symbols  / 40 bytes
    Blooper,  // ≤ 128 symbols / 80 bytes
    Bloopest, // ≤ 256 symbols / 160 bytes (impolite on LoRa)
    TooLong,  // > 256 symbols
}

impl SizeClass {
    pub fn name(&self) -> &'static str {
        match self {
            SizeClass::Blip => "blip",
            SizeClass::Bloop => "bloop",
            SizeClass::Blooper => "blooper",
            SizeClass::Bloopest => "bloopest",
            SizeClass::TooLong => "too long",
        }
    }

    pub fn is_polite_lora(&self) -> bool {
        matches!(self, SizeClass::Blip | SizeClass::Bloop)
    }
}

pub fn size_class(symbol_count: usize) -> SizeClass {
    match symbol_count {
        0..=16 => SizeClass::Blip,
        17..=64 => SizeClass::Bloop,
        65..=128 => SizeClass::Blooper,
        129..=256 => SizeClass::Bloopest,
        _ => SizeClass::TooLong,
    }
}

/// Validate a text string. Reports unsupported characters, then delegates structural
/// checks to `validate_symbols` on the valid symbols that were extracted.
pub fn validate_text(input: &str) -> ValidationReport {
    let mut report = ValidationReport::new();
    let mut symbols: Vec<Symbol> = Vec::new();

    for (i, c) in input.chars().enumerate() {
        match Symbol::try_from(c) {
            Ok(sym) => symbols.push(sym),
            Err(_) => report.add(
                ValidationIssueKind::UnsupportedCharacter,
                Some(i),
                format!("unsupported character {:?} at position {}", c, i),
            ),
        }
    }

    // Merge structural issues from the symbol stream.
    let structural = validate_symbols(&symbols);
    for issue in structural.issues {
        report.add(issue.kind, issue.position, issue.message);
    }

    report
}

/// Validate a symbol stream for structural issues: escape sequences, pipe balance,
/// unknown extension banks, and message length.
pub fn validate_symbols(symbols: &[Symbol]) -> ValidationReport {
    let mut report = ValidationReport::new();

    // Length check.
    if matches!(size_class(symbols.len()), SizeClass::TooLong) {
        report.add(
            ValidationIssueKind::MessageTooLong,
            None,
            format!(
                "message is {} symbols, exceeds bloopest limit of 256",
                symbols.len()
            ),
        );
    }

    // Pipe balance.
    let pipe_count = symbols.iter().filter(|&&s| s == Symbol::Pipe).count();
    if pipe_count % 2 != 0 {
        report.add(
            ValidationIssueKind::UnbalancedPipe,
            None,
            format!("odd number of pipes ({}); pipes should come in pairs", pipe_count),
        );
    }

    // Escape structure and bank checks.
    let mut i = 0;
    while i < symbols.len() {
        if symbols[i] == Symbol::Esc {
            if i + 1 >= symbols.len() {
                report.add(
                    ValidationIssueKind::DanglingEscape,
                    Some(i),
                    format!("dangling escape at symbol position {}", i),
                );
                break;
            }
            if i + 2 >= symbols.len() {
                report.add(
                    ValidationIssueKind::IncompleteEscape,
                    Some(i),
                    format!(
                        "incomplete escape at symbol position {} (bank present, index missing)",
                        i
                    ),
                );
                break;
            }
            let bank = symbols[i + 1] as u8;
            // Banks 0-7 have defined purposes; 8-30 are reserved; 31 is experimental.
            if bank >= 8 {
                report.add(
                    ValidationIssueKind::UnknownExtensionBank,
                    Some(i),
                    format!("unknown extension bank {} at position {}", bank, i),
                );
            }
            i += 3;
        } else {
            i += 1;
        }
    }

    report
}
