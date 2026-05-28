# Bloopnet / libBloop Project Handoff Prompt

You are Claude Code. Your job is to help create the initial implementation of **Bloopnet**, one step at a time.

Bloopnet is like Twitter, only better because it is worse.

Bloopnet deals in **Bloops**: short, fixed-width, 5-bit-per-character encoded text messages. The system is intentionally constrained, low-bandwidth, poetic, radio-friendly, and resistant to the feature creep of modern social platforms.

The first project goal is **libBloop**: a small, well-tested protocol and codec library that can become the foundation for future Bloopnet clients, command-line tools, LoRa radio gateways, BLE broadcasters, and web interfaces.

Do not build a full social network yet.

Build the protocol kernel first.

---

## Core Philosophy

Bloopnet should feel like:

- a tiny social radio protocol
- a constrained literary instrument
- a worse-but-better microblogging system
- a packet-friendly message format
- a thing that could be rendered on LEDs, punch tape, terminal screens, badges, or LoRa nodes

Important principles:

1. **The base language must remain small and stable.**
2. **The base encoding must be human-memorable.**
3. **Extensions are allowed, but should cost bandwidth.**
4. **Do not add modern social-media features prematurely.**
5. **No likes, quote-posts, algorithmic ranking, or rich profiles yet.**
6. **Start with encoding, decoding, validation, inspection, tests, and a CLI.**
7. **Transport metadata should be separate from the pure Bloop payload.**

The project should preserve the feeling that a Bloop is a small transmission, not a post optimized for engagement.

---

## Base 5-Bit Encoding

Each base symbol is exactly 5 bits.

There are 32 possible values:

| Value | Symbol | Meaning |
|---:|---|---|
| 0 | ` ` | space |
| 1 | `a` | lowercase a |
| 2 | `b` | lowercase b |
| 3 | `c` | lowercase c |
| 4 | `d` | lowercase d |
| 5 | `e` | lowercase e |
| 6 | `f` | lowercase f |
| 7 | `g` | lowercase g |
| 8 | `h` | lowercase h |
| 9 | `i` | lowercase i |
| 10 | `j` | lowercase j |
| 11 | `k` | lowercase k |
| 12 | `l` | lowercase l |
| 13 | `m` | lowercase m |
| 14 | `n` | lowercase n |
| 15 | `o` | lowercase o |
| 16 | `p` | lowercase p |
| 17 | `q` | lowercase q |
| 18 | `r` | lowercase r |
| 19 | `s` | lowercase s |
| 20 | `t` | lowercase t |
| 21 | `u` | lowercase u |
| 22 | `v` | lowercase v |
| 23 | `w` | lowercase w |
| 24 | `x` | lowercase x |
| 25 | `y` | lowercase y |
| 26 | `z` | lowercase z |
| 27 | `-` | dash |
| 28 | `?` | question mark |
| 29 | `|` | pipe / symmetrical parenthetical marker |
| 30 | `\n` | return / sentence break |
| 31 | `ESC` | escape / extension prefix |

### Notes on Design

- `0 = space` is intentional. Zero means silence, blankness, pause, or padding.
- `1–26 = a-z` is intentionally obvious and hand-decodable.
- `30 = RETURN` and `31 = ESC` place control/meta symbols at the top of the range.
- `?` is primitive because uncertainty and questioning are first-class Bloop behaviors.
- `!`, `.`, `,`, apostrophe, quotation marks, emoji, formatting, etc. should be extension-bank symbols, not base symbols.
- Numbers are not primitive. They should be written out as words.
- Uppercase should not be supported in the base set.

---

## ESC Extension Model

The `ESC` symbol introduces an extension token.

An extension token is encoded as three 5-bit symbols:

```text
ESC BANK INDEX
```

Where:

- `ESC` is symbol value `31`
- `BANK` is another 5-bit value, `0–31`
- `INDEX` is another 5-bit value, `0–31`

This gives:

```text
32 banks × 32 entries = 1024 possible extension symbols
```

Each extension token costs 15 bits total.

This is intentional. Punctuation, decoration, formatting, and future affordances should have a payload cost.

### Early Bank Convention

Do not fully implement all banks yet, but reserve the following conceptual map:

| Bank | Purpose |
|---:|---|
| 0 | canonical punctuation extensions |
| 1 | emotional / tonal markers |
| 2 | formatting / display hints |
| 3 | protocol/system annotations |
| 4 | pictograms / tiny glyphs |
| 5 | music/rhythm markers |
| 6 | geography/location hints |
| 7 | experimental art symbols |
| 8–30 | reserved |
| 31 | future meta-extension / experimental namespace |

For the first implementation, it is enough to parse and preserve extension tokens structurally. Do not design all glyphs yet.

### Malformed ESC Handling

The library should detect:

- dangling `ESC` at end of symbol stream
- `ESC BANK` without `INDEX`
- unsupported or unknown extension banks, depending on validation mode

Unknown extension tokens should not necessarily be fatal. A primitive client should be able to inspect or preserve them.

---

## Recommended Technical Architecture

Use a Rust core with TypeScript/WASM bindings.

### Why Rust?

Rust is appropriate because libBloop is about:

- bit packing
- deterministic parsing
- small binary formats
- strict validation
- embedded/radio friendliness
- future no_std support
- CLI tooling
- WASM bindings
- possible C/C++/firmware integration later

TypeScript is still important for UI, demos, and web clients, but the protocol kernel should live in Rust.

---

## Proposed Repository Layout

Start with a Rust workspace:

```text
libbloop/
  Cargo.toml
  README.md
  crates/
    bloop-core/
      Cargo.toml
      src/
        lib.rs
        symbol.rs
        token.rs
        codec.rs
        pack.rs
        error.rs
        validate.rs
      tests/
        codec_tests.rs
        pack_tests.rs
        validation_tests.rs
    bloop-packet/
      Cargo.toml
      src/
        lib.rs
        packet.rs
        crc.rs
      tests/
        packet_tests.rs
    bloop-cli/
      Cargo.toml
      src/
        main.rs
    bloop-wasm/
      Cargo.toml
      src/
        lib.rs
  js/
    bloop-js/
      package.json
      tsconfig.json
      src/
        index.ts
      README.md
  examples/
    web-terminal/
    packet-fuzzer/
    lora-gateway-skeleton/
```

Do not create every crate at once if that would make the first step too large. Prefer staged implementation.

---

## Phased Build Plan

Claude Code should implement the project in phases.

Each phase should include tests and a brief README update.

---

# Phase 1: Rust Workspace and bloop-core Basics

Create the Rust workspace and the `bloop-core` crate.

Implement:

- `Symbol` enum
- mapping from `char` to `Symbol`
- mapping from `Symbol` to `char`
- error type
- basic text-to-symbol conversion
- basic symbol-to-text conversion

### Required Rust API Sketch

```rust
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
```

Add conversions:

```rust
impl TryFrom<u8> for Symbol
impl From<Symbol> for u8
impl TryFrom<char> for Symbol
impl TryFrom<Symbol> for char
```

Note: `Esc` does not have a normal printable char mapping. It should be handled structurally.

### Phase 1 Tests

Test:

- `a` maps to `1`
- `z` maps to `26`
- space maps to `0`
- dash maps to `27`
- question mark maps to `28`
- pipe maps to `29`
- newline maps to `30`
- unsupported uppercase returns an error
- unsupported punctuation returns an error

---

# Phase 2: 5-Bit Packing and Unpacking

Implement pure bit-packing.

A list of symbols should pack into a byte array using consecutive 5-bit values.

Important: unpacking needs to know the symbol count, because the final byte may contain padding bits.

### Required API Sketch

```rust
pub fn pack_symbols(symbols: &[Symbol]) -> Vec<u8>;

pub fn unpack_symbols(bytes: &[u8], symbol_count: usize) -> Result<Vec<Symbol>, BloopError>;
```

Also expose convenience functions:

```rust
pub fn encode_text(input: &str) -> Result<EncodedBloop, BloopError>;

pub fn decode_text(bytes: &[u8], symbol_count: usize) -> Result<String, BloopError>;
```

`EncodedBloop` can be a small struct:

```rust
pub struct EncodedBloop {
    pub bytes: Vec<u8>,
    pub symbol_count: usize,
}
```

### Phase 2 Tests

Test round trips:

- empty string
- `hello`
- `hello world`
- `i saw |a dog|`
- `what?`
- multiline Bloop with `\n`

Also test known sizes:

```text
64 symbols = 320 bits = 40 bytes
128 symbols = 640 bits = 80 bytes
```

---

# Phase 3: Tokens and ESC Extension Parsing

Add a token layer above raw symbols.

### Required Token Model

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Symbol(Symbol),
    Escape { bank: u8, index: u8 },
}
```

Parsing rule:

```text
Esc, bank_symbol, index_symbol
```

The bank and index values are the raw `0–31` values of the following symbols.

### Required API Sketch

```rust
pub fn symbols_to_tokens(symbols: &[Symbol]) -> Result<Vec<Token>, BloopError>;

pub fn tokens_to_symbols(tokens: &[Token]) -> Result<Vec<Symbol>, BloopError>;
```

Important:

- `Escape { bank, index }` should round-trip exactly.
- Unknown bank/index combinations should still parse.
- Validation can warn about unsupported banks separately.

### Phase 3 Tests

Test:

- normal text tokens
- one escape token
- multiple escape tokens
- dangling `Esc` errors
- incomplete `Esc bank` errors
- `tokens_to_symbols` rejects bank/index > 31

---

# Phase 4: Validation and Inspection

Add validation utilities.

Validation should report issues without always being fatal.

### Suggested Validation Types

```rust
pub enum ValidationIssueKind {
    UnsupportedCharacter,
    DanglingEscape,
    IncompleteEscape,
    UnbalancedPipe,
    UnknownExtensionBank,
    MessageTooLong,
}

pub struct ValidationIssue {
    pub kind: ValidationIssueKind,
    pub position: Option<usize>,
    pub message: String,
}

pub struct ValidationReport {
    pub ok: bool,
    pub issues: Vec<ValidationIssue>,
}
```

### Pipe Validation

The pipe character `|` is used as a symmetrical parenthetical marker.

For now:

- even number of pipes = balanced
- odd number of pipes = unbalanced warning or error, depending on mode

Do not overcomplicate nested parsing yet.

### Message Length Validation

Support recommended size classes:

| Name | Symbols | Payload Bytes |
|---|---:|---:|
| blip | 16 | 10 |
| bloop | 64 | 40 |
| blooper | 128 | 80 |
| bloopest | 256 | 160 |

A standard Bloop should be 64 symbols.

A blooper should be 128 symbols.

A bloopest should be considered impolite for LoRa unless explicitly allowed.

---

# Phase 5: CLI

Create `bloop-cli`.

The CLI should provide:

```bash
bloop encode "hello world"
bloop decode --symbols 11 <hex-or-file>
bloop inspect "hello |small world|"
bloop pack input.txt -o message.blp
bloop unpack message.blp --symbols 64
```

Initial commands can be simpler if needed:

```bash
bloop encode "hello world"
bloop decode --hex "..." --symbols 11
bloop inspect "hello world"
```

### CLI Output Goals

The CLI should be charming and useful.

For inspection, show:

```text
text: hello world
symbols: 11
bits: 55
payload bytes: 7
hex: ...
valid: yes
```

For a 64-character Bloop, show:

```text
symbols: 64
bits: 320
payload bytes: 40
packet suitability: polite lora bloop
```

---

# Phase 6: Packet Layer

Create `bloop-packet`.

Do not pollute the pure Bloop payload with network metadata.

Define a simple packet wrapper suitable for LoRa, BLE, serial, and file transport.

### Draft Packet Structure

```text
magic:        2 bytes   "BL"
version:      1 byte
flags:        1 byte
sender_id:    4 bytes
packet_id:    4 bytes
symbol_count: 2 bytes
payload_len:  2 bytes
payload:      n bytes
crc16:        2 bytes
```

Approximate overhead before payload and CRC: 16 bytes.

A 64-character Bloop payload is 40 bytes, so a full packet is roughly 58 bytes.

That is a polite LoRa-sized target.

### Packet API Sketch

```rust
pub struct BloopPacket {
    pub version: u8,
    pub flags: u8,
    pub sender_id: u32,
    pub packet_id: u32,
    pub symbol_count: u16,
    pub payload: Vec<u8>,
}

pub fn encode_packet(packet: &BloopPacket) -> Result<Vec<u8>, PacketError>;

pub fn decode_packet(bytes: &[u8]) -> Result<BloopPacket, PacketError>;
```

Add CRC validation.

Use a simple CRC16 implementation or a small dependency, but document the choice.

---

# Phase 7: WASM and TypeScript Bindings

Create `bloop-wasm` using `wasm-bindgen`.

Then create a TypeScript wrapper package in `js/bloop-js`.

### Desired TypeScript API

```ts
import {
  encodeText,
  decodeText,
  inspectText,
  validateText,
} from "@bloopnet/libbloop";

const encoded = encodeText("hello world");
console.log(encoded.bytes);
console.log(encoded.symbolCount);

const decoded = decodeText(encoded.bytes, encoded.symbolCount);
```

### TypeScript Types

```ts
export interface EncodedBloop {
  bytes: Uint8Array;
  symbolCount: number;
}

export interface InspectionReport {
  text: string;
  symbols: number;
  bits: number;
  payloadBytes: number;
  valid: boolean;
  issues: ValidationIssue[];
}
```

---

# Phase 8: Minimal Web Terminal Example

Create a very small web example.

It should:

- accept Bloop text
- show validation issues
- show symbol count
- show bit count
- show packed byte count
- show hex
- decode the packed bytes back to text

Avoid building feeds, accounts, or timelines.

This is a protocol demonstrator.

---

# Phase 9: LoRa Gateway Skeleton

Create an example skeleton, not a full hardware integration.

Show how a Bloop packet might be sent over a serial-attached LoRa modem.

The example can be pseudo-real, but should be structured.

Possible CLI shape:

```bash
bloop transmit --port /dev/tty.usbserial "weather looks wrong"
bloop listen --port /dev/tty.usbserial
```

Document that real LoRa behavior depends on:

- region
- spreading factor
- bandwidth
- coding rate
- duty-cycle limits
- module firmware
- antenna
- local regulations

The recommended polite target is a 64-symbol Bloop payload, around 40 bytes before packet overhead.

---

## Important Implementation Constraints

Claude Code should follow these constraints:

1. Keep the first implementation small.
2. Write tests before or alongside implementation.
3. Prefer simple code over clever abstractions.
4. Do not build authentication yet.
5. Do not build a database yet.
6. Do not build social-network features yet.
7. Do not add Unicode support beyond the defined charset.
8. Do not silently normalize uppercase unless explicitly requested by an option.
9. Do not treat punctuation as free.
10. Do not make ESC extensions mandatory for ordinary readability.
11. Keep `bloop-core` free of UI assumptions.
12. Keep transport packet metadata separate from payload encoding.
13. Keep the packed 5-bit codec deterministic and well tested.

---

## Optional Normalization Mode

The strict default should reject unsupported characters.

Later, an optional normalization mode could:

- lowercase uppercase letters
- convert periods to ESC bank 0 index N
- convert commas to ESC bank 0 index N
- convert exclamation marks to ESC bank 0 index N
- convert tabs to spaces
- collapse repeated spaces

But strict mode should come first.

---

## Suggested README Framing

The README should say something like:

```markdown
# libBloop

libBloop is the reference codec and packet library for Bloopnet, a deliberately constrained low-bandwidth social messaging system.

A Bloop is a short 5-bit-per-symbol message using lowercase letters, spaces, minimal punctuation, returns, pipes, and an escape mechanism for future extension banks.

Bloopnet is like Twitter, only better because it is worse.
```

---

## Definition of Done for Initial Milestone

The first good milestone is complete when:

- `bloop-core` exists
- text can be encoded to 5-bit packed bytes
- packed bytes can be decoded back to text
- symbol count is tracked
- invalid characters are rejected
- ESC sequences can be represented structurally
- tests pass
- CLI can encode/decode/inspect a Bloop
- README explains the philosophy and basic encoding table

Do not continue to web UI, LoRa, or packets until this milestone is solid.

---

## Example Bloops

```text
hello world
```

```text
i saw |a very strange dog| today
```

```text
weather looks wrong
bring tea
```

```text
are you there?
```

```text
no numbers
write them out
```

---

## Future Ideas, Not Yet Implementation Requirements

These are useful directions, but should not distract from libBloop:

- BLE Bloop beacons
- LoRa Bloop repeaters
- store-and-forward gossip nodes
- terminal client
- Ravel/Web Components UI
- LED sign renderer
- Bloop printer
- gallery installation mode
- damaged/corrupted Bloop rendering
- optional extension-bank glyph fonts
- tiny pictogram bank
- audio/tone representation of Bloops
- QR-like visual Bloop glyphs
- local-first archive
- packet radio bridge

---

## The Most Important Instruction

Start with the codec.

Do not build Twitter.

Build the thing that can encode a tiny thought, transmit it politely, and decode it again.
