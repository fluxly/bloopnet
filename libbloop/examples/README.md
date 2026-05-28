# Examples

Two runnable examples ship with libbloop.

---

## Web terminal

A live browser inspector. Type a Bloop, see the symbol count, bit count,
payload bytes, hex encoding, size class, and decoded text in real time.

### Prerequisites

- [Node.js](https://nodejs.org/) 18+
- [Rust](https://rustup.rs/) + `wasm-pack` (`cargo install wasm-pack`)

### Run

```bash
# 1. Build the WASM bindings
wasm-pack build crates/bloop-wasm --target bundler \
  --out-dir ../../examples/web-terminal/wasm

# 2. Install JS deps and start the dev server
cd examples/web-terminal
npm install
npm run dev
# open http://localhost:5173
```

> The `wasm-pack` step only needs to re-run when you change Rust code in
> `crates/bloop-core` or `crates/bloop-wasm`.

---

## LoRa gateway skeleton

A structured example showing how Bloop packets flow over a serial-attached
LoRa modem. Runs fully in simulation mode — no hardware required.

### Prerequisites

- [Rust](https://rustup.rs/)

### Run (simulation)

```bash
# From the libbloop/ workspace root

# Encode and transmit a bloop (prints packet fields, hex, air-time estimates)
cargo run -p bloop-lora-gateway -- transmit --simulate "weather looks wrong"

# Listen for incoming bloops (plays back a canned example)
cargo run -p bloop-lora-gateway -- listen --simulate
```

### Run (real hardware)

```bash
cargo run -p bloop-lora-gateway -- transmit \
  --port /dev/tty.usbserial-0001 "weather looks wrong"

cargo run -p bloop-lora-gateway -- listen \
  --port /dev/tty.usbserial-0001
```

Replace `/dev/tty.usbserial-0001` with your modem's serial device. Most LoRa
modules need AT-command configuration before entering transparent serial mode —
consult your module's datasheet.

See [`lora-gateway-skeleton/README.md`](lora-gateway-skeleton/README.md) for
wire format details and LoRa air-time estimates.
