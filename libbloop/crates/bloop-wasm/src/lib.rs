use bloop_core::validate::ValidationIssueKind;
use bloop_core::{codec, symbol::Symbol, validate};
use serde::Serialize;
use wasm_bindgen::prelude::*;

// ── EncodedBloop ─────────────────────────────────────────────────────────────

/// Returned by `encodeText`. The `bytes` getter produces a `Uint8Array`.
#[wasm_bindgen]
pub struct EncodedBloop {
    bytes: Vec<u8>,
    symbol_count: usize,
}

#[wasm_bindgen]
impl EncodedBloop {
    #[wasm_bindgen(getter)]
    pub fn bytes(&self) -> Box<[u8]> {
        self.bytes.clone().into_boxed_slice()
    }

    #[wasm_bindgen(getter, js_name = symbolCount)]
    pub fn symbol_count(&self) -> usize {
        self.symbol_count
    }
}

// ── Serializable report types ─────────────────────────────────────────────────

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct WasmValidationIssue {
    kind: &'static str,
    position: Option<usize>,
    message: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct WasmValidationReport {
    ok: bool,
    issues: Vec<WasmValidationIssue>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct WasmInspectionReport {
    text: String,
    symbols: usize,
    bits: usize,
    payload_bytes: usize,
    valid: bool,
    issues: Vec<WasmValidationIssue>,
    size_class: String,
    lora_polite: bool,
}

fn kind_str(kind: &ValidationIssueKind) -> &'static str {
    match kind {
        ValidationIssueKind::UnsupportedCharacter => "UnsupportedCharacter",
        ValidationIssueKind::DanglingEscape => "DanglingEscape",
        ValidationIssueKind::IncompleteEscape => "IncompleteEscape",
        ValidationIssueKind::UnbalancedPipe => "UnbalancedPipe",
        ValidationIssueKind::UnknownExtensionBank => "UnknownExtensionBank",
        ValidationIssueKind::MessageTooLong => "MessageTooLong",
    }
}

fn to_wasm_issues(
    issues: &[bloop_core::validate::ValidationIssue],
) -> Vec<WasmValidationIssue> {
    issues
        .iter()
        .map(|i| WasmValidationIssue {
            kind: kind_str(&i.kind),
            position: i.position,
            message: i.message.clone(),
        })
        .collect()
}

// ── Exported functions ────────────────────────────────────────────────────────

/// Encode a Bloop text string to packed 5-bit bytes.
#[wasm_bindgen(js_name = encodeText)]
pub fn encode_text(input: &str) -> Result<EncodedBloop, JsError> {
    let enc = codec::encode_text(input).map_err(|e| JsError::new(&e.to_string()))?;
    Ok(EncodedBloop {
        bytes: enc.bytes,
        symbol_count: enc.symbol_count,
    })
}

/// Decode packed bytes back to text. `symbol_count` must match the value used during encoding.
///
/// Exported as `decodeBloopText` to avoid a name collision with wasm-bindgen's
/// internal `decodeText` UTF-8 helper in the generated `_bg.js` glue file.
#[wasm_bindgen(js_name = decodeBloopText)]
pub fn decode_text(bytes: &[u8], symbol_count: usize) -> Result<String, JsError> {
    codec::decode_text(bytes, symbol_count).map_err(|e| JsError::new(&e.to_string()))
}

/// Validate a Bloop text string and return a `ValidationReport`.
#[wasm_bindgen(js_name = validateText)]
pub fn validate_text(input: &str) -> Result<JsValue, JsError> {
    let report = validate::validate_text(input);
    let result = WasmValidationReport {
        ok: report.ok,
        issues: to_wasm_issues(&report.issues),
    };
    serde_wasm_bindgen::to_value(&result).map_err(|e| JsError::new(&e.to_string()))
}

/// Inspect a Bloop text string: symbol count, bit count, size class, and validation.
#[wasm_bindgen(js_name = inspectText)]
pub fn inspect_text(input: &str) -> Result<JsValue, JsError> {
    let report = validate::validate_text(input);

    let symbol_count: usize = input
        .chars()
        .filter(|&c| Symbol::try_from(c).is_ok())
        .count();
    let bits = symbol_count * 5;
    let payload_bytes = (bits + 7) / 8;
    let sc = validate::size_class(symbol_count);

    let result = WasmInspectionReport {
        text: input.to_string(),
        symbols: symbol_count,
        bits,
        payload_bytes,
        valid: report.ok,
        issues: to_wasm_issues(&report.issues),
        size_class: sc.name().to_string(),
        lora_polite: sc.is_polite_lora(),
    };
    serde_wasm_bindgen::to_value(&result).map_err(|e| JsError::new(&e.to_string()))
}

// ── Tests ─────────────────────────────────────────────────────────────────────

// Host-target tests: verify pure-Rust logic without touching the JS environment.
#[cfg(test)]
mod host_tests {
    use bloop_core::{codec, validate};

    #[test]
    fn encode_decode_round_trip() {
        let enc = codec::encode_text("hello world").unwrap();
        assert_eq!(enc.symbol_count, 11);
        assert_eq!(enc.bytes.len(), 7);
        let text = codec::decode_text(&enc.bytes, enc.symbol_count).unwrap();
        assert_eq!(text, "hello world");
    }

    #[test]
    fn encode_unsupported_char_returns_error() {
        assert!(codec::encode_text("Hello").is_err());
    }

    #[test]
    fn validate_valid_text_is_ok() {
        let report = validate::validate_text("hello world");
        assert!(report.ok);
    }

    #[test]
    fn inspect_symbol_count() {
        use bloop_core::symbol::Symbol;
        let count: usize = "hello world"
            .chars()
            .filter(|&c| Symbol::try_from(c).is_ok())
            .count();
        assert_eq!(count, 11);
    }
}

// WASM-target tests: run in browser/node via `wasm-pack test`.
#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use super::*;
    use wasm_bindgen_test::*;
    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn wasm_encode_decode_round_trip() {
        let enc = encode_text("hello world").unwrap();
        assert_eq!(enc.symbol_count(), 11);
        let text = decode_text(&enc.bytes(), enc.symbol_count()).unwrap();
        assert_eq!(text, "hello world");
    }

    #[wasm_bindgen_test]
    fn wasm_encode_rejects_uppercase() {
        assert!(encode_text("Hello").is_err());
    }

    #[wasm_bindgen_test]
    fn wasm_validate_returns_value() {
        let _ = validate_text("hello world").unwrap();
    }

    #[wasm_bindgen_test]
    fn wasm_inspect_returns_value() {
        let _ = inspect_text("hello world").unwrap();
    }
}
