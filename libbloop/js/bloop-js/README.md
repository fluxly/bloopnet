# @bloopnet/libbloop

TypeScript bindings for the Bloopnet codec library.

## Build

WASM must be built before the TypeScript package:

```bash
# 1. Install wasm-pack if you don't have it
cargo install wasm-pack

# 2. Build the WASM package
npm run build:wasm

# 3. Build the TypeScript wrapper
npm run build
```

## Usage

```ts
import { encodeText, decodeText, inspectText, validateText } from "@bloopnet/libbloop";

// Encode text to packed 5-bit bytes
const encoded = encodeText("hello world");
console.log(encoded.bytes);       // Uint8Array
console.log(encoded.symbolCount); // 11

// Decode back to text
const text = decodeText(encoded.bytes, encoded.symbolCount);
// → "hello world"

// Inspect
const report = inspectText("hello world");
// → { text: "hello world", symbols: 11, bits: 55, payloadBytes: 7,
//     valid: true, issues: [], sizeClass: "blip", loraPolite: true }

// Validate
const v = validateText("Hello World!");
// → { ok: false, issues: [{ kind: "UnsupportedCharacter", ... }, ...] }
```

## Notes

- Only lowercase `a–z`, space, `-`, `?`, `|`, and `\n` are supported in the base charset.
- Numbers must be written out as words.
- Uppercase, digits, and most punctuation are rejected unless accessed via ESC extension tokens.
