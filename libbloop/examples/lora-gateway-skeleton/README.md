# bloop-lora-gateway

LoRa gateway skeleton for Bloopnet.

This is a structured example showing how Bloop packets flow over a serial-attached
LoRa modem. It is not a finished hardware driver. Use `--simulate` to run without
a modem attached.

## Usage

```bash
# Encode and transmit a bloop (simulation)
bloop-lora transmit --simulate "weather looks wrong"

# Transmit on real hardware
bloop-lora transmit --port /dev/tty.usbserial-0001 "weather looks wrong"

# Listen for incoming bloops (simulation — plays back a canned example)
bloop-lora listen --simulate

# Listen on real hardware (blocks until Ctrl-C)
bloop-lora listen --port /dev/tty.usbserial-0001
```

## Wire format

Each transmission is a `BloopPacket` written as raw bytes over the UART:

```
magic:        2 bytes  "BL"
version:      1 byte
flags:        1 byte
sender_id:    4 bytes  big-endian u32
packet_id:    4 bytes  big-endian u32
symbol_count: 2 bytes  big-endian u16
payload_len:  2 bytes  big-endian u16
payload:      n bytes  packed 5-bit Bloop symbols
crc16:        2 bytes  CRC-16/CCITT-FALSE over all preceding bytes
```

A standard 64-symbol Bloop produces a 58-byte wire frame — a polite LoRa payload size.

## LoRa air time

Actual air time depends on your configuration. Rough estimates for a 58-byte packet:

| SF  | BW      | CR  | Air time  | Notes                    |
|-----|---------|-----|-----------|--------------------------|
| SF7 | 125 kHz | 4/5 | ~99 ms    | Fast, short range        |
| SF9 | 125 kHz | 4/5 | ~329 ms   | Balanced                 |
| SF12| 125 kHz | 4/5 | ~2465 ms  | Long range, very slow    |

A bloopest (256-symbol Bloop, 160 payload bytes, ~176 wire bytes) at SF12 exceeds 7
seconds of air time and is **impolite on shared LoRa networks**. The recommended
maximum for shared use is a 64-symbol Bloop.

## Real LoRa behavior depends on

- **Region** — frequency band, EIRP limits, dwell time (EU868, US915, AS923, …)
- **Spreading factor** — SF7 (fastest) through SF12 (longest range)
- **Bandwidth** — 125 kHz / 250 kHz / 500 kHz
- **Coding rate** — 4/5 through 4/8
- **Duty-cycle limits** — typically 1% in EU868; check local regulations
- **Module firmware** — AT-command set varies by vendor (RYLR998, RFM95, SX1262, …)
- **Antenna** — matched to frequency and impedance
- **Local regulations** — ISM band rules differ by jurisdiction

This skeleton uses raw binary UART output. Most LoRa modules require AT-command
configuration before entering transparent (raw) serial mode. Consult your module's
datasheet for the exact command sequence.

## Hardware tested with

None — this is a skeleton. Contributions welcome.
