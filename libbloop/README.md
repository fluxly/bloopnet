# libBloop

libBloop is the reference codec and packet library for Bloopnet, a deliberately constrained low-bandwidth social messaging system.

A Bloop is a short 5-bit-per-symbol message using lowercase letters, spaces, minimal punctuation, returns, pipes, and an escape mechanism for future extension banks.

**Bloopnet is like Twitter, only better because it is worse.**

---

## Philosophy

Bloopnet should feel like:

- a tiny social radio protocol
- a constrained literary instrument
- a worse-but-better microblogging system
- a packet-friendly message format
- a thing that could be rendered on LEDs, punch tape, terminal screens, badges, or LoRa nodes

Core principles:

1. The base language must remain small and stable.
2. The base encoding must be human-memorable.
3. Extensions are allowed, but should cost bandwidth.
4. A Bloop is a small transmission, not a post optimized for engagement.
5. Transport metadata is separate from the pure Bloop payload.

---

## Base 5-Bit Encoding

Each symbol is exactly 5 bits. There are 32 possible values:

| Value | Symbol | Meaning |
|---:|:---:|---|
| 0 | ` ` | space |
| 1–26 | `a`–`z` | lowercase letters |
| 27 | `-` | dash |
| 28 | `?` | question mark |
| 29 | `\|` | pipe / symmetrical parenthetical |
| 30 | `\n` | return / sentence break |
| 31 | `ESC` | escape / extension prefix |

Design notes:

- `0 = space` is intentional. Zero means silence, blankness, pause, or padding.
- `1–26 = a-z` is intentionally obvious and hand-decodable.
- Numbers are not primitive — write them out as words.
- Uppercase is not supported in the base set.
- `!`, `.`, `,`, apostrophes, emoji, and formatting are extension-bank symbols.
- `?` is first-class. Uncertainty and questioning are native Bloop behaviors.

---

## ESC Extension Model

`ESC` (value 31) introduces a 3-symbol extension token:

```
ESC  BANK  INDEX
```

- `BANK` and `INDEX` are each a 5-bit value (0–31)
- 32 banks × 32 entries = **1024 possible extension symbols**
- Each extension token costs **15 bits** — punctuation and decoration have a payload cost

### Bank map

| Bank | Purpose |
|---:|---|
| 0 | canonical punctuation extensions |
| 1 | emotional / tonal markers |
| 2 | formatting / display hints |
| 3 | protocol / system annotations |
| 4 | pictograms / tiny glyphs |
| 5 | music / rhythm markers |
| 6 | geography / location hints |
| 7 | experimental art symbols |
| 8–30 | reserved |
| 31 | future meta-extension / experimental namespace |

Unknown extension tokens are preserved structurally and do not cause parse failures.

---

## Size Classes

| Name | Symbols | Payload Bytes | Notes |
|---|---:|---:|---|
| pulse | 16 | 10 | |
| bloop | 64 | 40 | standard; polite LoRa target |
| longbloop | 128 | 80 | pushing LoRa limits |
| flood | 256 | 160 | impolite on shared LoRa networks |

A standard Bloop is 64 symbols / 40 payload bytes. A full 64-symbol BloopPacket (with 16-byte header and 2-byte CRC) is **58 bytes** on the wire — a polite LoRa-sized target.

---

## Example Bloops

```
hello world
```

```
i saw |a very strange dog| today
```

```
weather looks wrong
bring tea
```

```
are you there?
```

```
no numbers
write them out
```

---

## Repository Layout

```
libbloop/
  crates/
    bloop-core/       symbol enum, 5-bit codec, token layer, validation
    bloop-packet/     packet framing, CRC-16, wire format
    bloop-cli/        bloop encode / decode / inspect
    bloop-wasm/       wasm-bindgen bindings (built with wasm-pack)
  js/
    bloop-js/         TypeScript wrapper — @bloopnet/libbloop
  examples/
    web-terminal/     live browser inspector (Vite + WASM)
    lora-gateway-skeleton/  serial transmit/listen skeleton
```

---

## Building

```bash
cargo build
cargo test
```

### CLI

```bash
cargo build -p bloop-cli

bloop encode "hello world"
bloop decode --hex 4158c782ef9308 --symbols 11
bloop inspect "i saw |a very strange dog| today"
```

### WASM / TypeScript

```bash
cargo install wasm-pack

wasm-pack build crates/bloop-wasm --target bundler \
  --out-dir ../../examples/web-terminal/wasm

cd js/bloop-js && npm run build:wasm && npm run build
```

### Web terminal

```bash
cd examples/web-terminal
npm install && npm run dev
# open http://localhost:5173
```

### LoRa gateway skeleton

```bash
# Simulate without hardware
cargo run -p bloop-lora-gateway -- transmit --simulate "weather looks wrong"
cargo run -p bloop-lora-gateway -- listen --simulate

# Real hardware
cargo run -p bloop-lora-gateway -- transmit --port /dev/tty.usbserial-0001 "weather looks wrong"
cargo run -p bloop-lora-gateway -- listen --port /dev/tty.usbserial-0001
```

---

## Crates

### bloop-core

The protocol kernel. No UI assumptions, no transport metadata.

- `Symbol` — 5-bit value enum with `TryFrom<char>`, `TryFrom<u8>`, `From<Symbol> for u8`, `TryFrom<Symbol> for char`
- `text_to_symbols` / `symbols_to_text` — text ↔ symbol stream
- `pack_symbols` / `unpack_symbols` — symbol stream ↔ packed bytes
- `encode_text` / `decode_text` — text ↔ `EncodedBloop { bytes, symbol_count }`
- `symbols_to_tokens` / `tokens_to_symbols` — parse ESC extension triples
- `validate_text` / `validate_symbols` — non-fatal `ValidationReport` with issue list
- `size_class` — classify symbol count as pulse / bloop / longbloop / flood / too long

### bloop-packet

Transport framing. No dependency on bloop-core in production code.

- `BloopPacket` — `version`, `flags`, `sender_id`, `packet_id`, `symbol_count`, `payload`
- `encode_packet` / `decode_packet` — serialize/deserialize the wire format
- `crc16` — CRC-16/CCITT-FALSE (poly `0x1021`, init `0xFFFF`)

### bloop-wasm

wasm-bindgen bindings. Built separately with `wasm-pack`; excluded from the default workspace build.

Exports: `encodeText`, `decodeBloopText`, `validateText`, `inspectText`

### bloop-cli

```
bloop encode <text>
bloop decode --hex <hex> --symbols <n>
bloop inspect <text>
```

---

## Packet wire format

```
magic:        2 bytes   "BL"
version:      1 byte
flags:        1 byte
sender_id:    4 bytes   big-endian u32
packet_id:    4 bytes   big-endian u32
symbol_count: 2 bytes   big-endian u16
payload_len:  2 bytes   big-endian u16
payload:      n bytes
crc16:        2 bytes   CRC-16/CCITT-FALSE over all preceding bytes
```

---

## Validation

`validate_text` and `validate_symbols` return a `ValidationReport` with a list of `ValidationIssue` entries. Issues never cause a panic — the consumer decides what is fatal.

| Issue kind | Meaning |
|---|---|
| `UnsupportedCharacter` | char has no base-symbol mapping |
| `DanglingEscape` | `ESC` at end of stream with no following symbols |
| `IncompleteEscape` | `ESC BANK` with no `INDEX` |
| `UnbalancedPipe` | odd number of pipe characters |
| `UnknownExtensionBank` | bank ≥ 8 (reserved or experimental) |
| `MessageTooLong` | more than 256 symbols |

---

## Future directions

- BLE Bloop beacons
- LoRa Bloop repeaters and store-and-forward gossip nodes
- Terminal client
- LED sign renderer
- Bloop printer / punch tape output
- Audio / tone representation
- QR-style visual Bloop glyphs
- Local-first archive
- Packet radio bridge
- Extension bank glyph fonts (punctuation, pictograms, music markers)
- Damaged / corrupted Bloop rendering
