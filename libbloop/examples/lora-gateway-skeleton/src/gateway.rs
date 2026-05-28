use bloop_core::codec::{decode_text, encode_text};
use bloop_core::validate::{size_class, validate_text};
use bloop_packet::{decode_packet, encode_packet, BloopPacket, MAGIC, VERSION};
use std::io::{Read, Write};
use std::time::Duration;

// ── Air-time estimation ───────────────────────────────────────────────────────

/// Estimate LoRa packet air time in milliseconds using the standard Semtech
/// formula. Assumes explicit header, CRC enabled, 8-symbol preamble.
///
/// Parameters:
///   payload_bytes — total wire bytes including packet header and CRC
///   sf            — spreading factor (7–12)
///   bw_hz         — bandwidth in Hz (e.g. 125_000)
///   cr            — coding rate (1 = 4/5, 2 = 4/6, 3 = 4/7, 4 = 4/8)
pub fn air_time_ms(payload_bytes: usize, sf: u8, bw_hz: f64, cr: u8) -> f64 {
    let sf = sf as f64;
    let cr = cr as f64;
    // Low data rate optimisation is required for SF11/SF12 at BW 125 kHz.
    let de: f64 = if sf >= 11.0 && bw_hz <= 125_000.0 { 1.0 } else { 0.0 };

    let t_sym = 2f64.powf(sf) / bw_hz; // seconds per symbol
    let t_preamble = (8.0 + 4.25) * t_sym;

    // Payload symbols (CRC=1, implicit header=0)
    let num = (8.0 * payload_bytes as f64 - 4.0 * sf + 28.0 + 16.0).max(0.0);
    let denom = 4.0 * (sf - 2.0 * de);
    let payload_syms = 8.0 + (num / denom).ceil() * (cr + 4.0);

    (t_preamble + payload_syms * t_sym) * 1_000.0
}

fn air_time_table(wire_bytes: usize) {
    println!("  estimated air time ({} wire bytes):", wire_bytes);
    for (sf, label) in [(7u8, "fast / short range"), (9, "balanced"), (12, "slow / long range")] {
        let ms = air_time_ms(wire_bytes, sf, 125_000.0, 1);
        println!("    SF{sf:<2}  BW125  CR4/5   {:>7.0} ms    ({label})", ms);
    }
    println!();
    println!("  note: real airtime depends on region, duty-cycle limits, antenna,");
    println!("        module firmware, spreading factor, bandwidth, and coding rate.");
}

// ── Helpers ───────────────────────────────────────────────────────────────────

pub fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn print_packet_fields(packet: &BloopPacket) {
    println!("  magic:         BL");
    println!("  version:       {}", packet.version);
    println!("  flags:         {:#04x}", packet.flags);
    println!("  sender_id:     {:#010x}", packet.sender_id);
    println!("  packet_id:     {:#010x}", packet.packet_id);
    println!("  symbol_count:  {}", packet.symbol_count);
    println!("  payload_len:   {}", packet.payload.len());
}

fn decode_and_display(wire: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    let packet = decode_packet(wire)?;
    println!("packet fields:");
    print_packet_fields(&packet);
    println!("  crc16:         ok");

    let text = decode_text(&packet.payload, packet.symbol_count as usize)?;
    println!();
    println!("decoded text:");
    println!("  {text}");
    Ok(())
}

/// Scan a buffer for the "BL" magic bytes and return the offset.
fn find_magic(buf: &[u8]) -> Option<usize> {
    buf.windows(2).position(|w| w == MAGIC)
}

// ── Transmit ──────────────────────────────────────────────────────────────────

pub struct TransmitArgs {
    pub text: String,
    pub port: String,
    pub baud: u32,
    pub sender_id: u32,
    pub packet_id: u32,
    pub simulate: bool,
}

pub fn cmd_transmit(args: &TransmitArgs) -> Result<(), Box<dyn std::error::Error>> {
    // Validate first.
    let report = validate_text(&args.text);
    if !report.ok {
        eprintln!("validation failed:");
        for issue in &report.issues {
            eprintln!("  ! {}", issue.message);
        }
        return Err("bloop did not pass validation".into());
    }

    // Encode.
    let enc = encode_text(&args.text)?;
    let sc = size_class(enc.symbol_count);
    println!("encoding...");
    println!("  text:          {}", args.text);
    println!("  symbols:       {}", enc.symbol_count);
    println!("  bits:          {}", enc.symbol_count * 5);
    println!("  payload bytes: {}", enc.bytes.len());
    println!("  size class:    {}  (lora polite: {})",
        sc.name(),
        if sc.is_polite_lora() { "yes" } else { "no" });
    println!();

    // Wrap in packet.
    let packet = BloopPacket {
        version: VERSION,
        flags: 0,
        sender_id: args.sender_id,
        packet_id: args.packet_id,
        symbol_count: enc.symbol_count as u16,
        payload: enc.bytes,
    };
    let wire = encode_packet(&packet)?;

    println!("packet...");
    print_packet_fields(&packet);
    println!("  total bytes:   {}", wire.len());
    println!("  hex:           {}", to_hex(&wire));
    println!();

    air_time_table(wire.len());

    // Transmit.
    if args.simulate {
        println!("[simulate] would write {} bytes to {} at {} baud",
            wire.len(), args.port, args.baud);
        println!("[simulate] done");
    } else {
        println!("opening {}  at {} baud...", args.port, args.baud);
        let mut port = serialport::new(&args.port, args.baud)
            .timeout(Duration::from_secs(5))
            .open()?;
        port.write_all(&wire)?;
        println!("transmitted {} bytes", wire.len());
    }

    Ok(())
}

// ── Listen ────────────────────────────────────────────────────────────────────

pub struct ListenArgs {
    pub port: String,
    pub baud: u32,
    pub simulate: bool,
}

pub fn cmd_listen(args: &ListenArgs) -> Result<(), Box<dyn std::error::Error>> {
    if args.simulate {
        println!("[simulate] listening on {} at {} baud...", args.port, args.baud);
        println!("[simulate] playing back a pre-encoded bloop");
        println!();

        let enc = encode_text("weather looks wrong")?;
        let packet = BloopPacket {
            version: VERSION,
            flags: 0,
            sender_id: 0xBEEF0001,
            packet_id: 1,
            symbol_count: enc.symbol_count as u16,
            payload: enc.bytes,
        };
        let wire = encode_packet(&packet)?;

        println!("received {} bytes", wire.len());
        println!("hex:           {}", to_hex(&wire));
        println!();
        decode_and_display(&wire)?;
        return Ok(());
    }

    println!("listening on {} at {} baud...", args.port, args.baud);
    println!("waiting for bloops — Ctrl-C to stop");
    println!();

    let mut port = serialport::new(&args.port, args.baud)
        .timeout(Duration::from_millis(500))
        .open()?;

    let mut buf: Vec<u8> = Vec::new();

    loop {
        let mut byte = [0u8; 1];
        match port.read(&mut byte) {
            Ok(0) => {}
            Ok(_) => {
                buf.push(byte[0]);

                // Trim everything before the first "BL" magic.
                if let Some(start) = find_magic(&buf) {
                    if start > 0 {
                        buf.drain(..start);
                    }
                } else {
                    // No magic found yet; keep only the last byte in case
                    // it's the 'B' of an incoming "BL".
                    buf.retain(|_| false);
                    buf.push(byte[0]);
                    continue;
                }

                // Need at least the 18-byte minimum packet size.
                if buf.len() < 18 {
                    continue;
                }

                // Peek at payload_len to know how many bytes to wait for.
                let payload_len = u16::from_be_bytes([buf[14], buf[15]]) as usize;
                let needed = 18 + payload_len;
                if buf.len() < needed {
                    continue;
                }

                let frame = buf[..needed].to_vec();
                buf.drain(..needed);

                println!("received {} bytes", frame.len());
                println!("hex:           {}", to_hex(&frame));
                println!();

                match decode_and_display(&frame) {
                    Ok(_) => {}
                    Err(e) => eprintln!("  decode error: {e}"),
                }
                println!();
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {}
            Err(e) => return Err(e.into()),
        }
    }
}
