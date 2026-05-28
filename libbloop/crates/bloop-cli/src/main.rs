use bloop_core::codec::{decode_text, encode_text};
use bloop_core::symbol::Symbol;
use bloop_core::validate::{size_class, validate_text, SizeClass};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "bloop",
    about = "Bloopnet codec — encode, decode, and inspect Bloops\n\nBloopnet is like Twitter, only better because it is worse."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Encode text to packed 5-bit bytes
    Encode {
        /// The text to encode (lowercase, spaces, - ? | \\n only)
        text: String,
    },
    /// Decode a packed Bloop from hex
    Decode {
        /// Packed bytes as a hex string
        #[arg(long)]
        hex: String,
        /// Number of symbols encoded in the payload
        #[arg(long)]
        symbols: usize,
    },
    /// Inspect a text Bloop: symbol count, bit count, size class, and validation
    Inspect {
        /// The text to inspect
        text: String,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Encode { text } => cmd_encode(&text),
        Commands::Decode { hex, symbols } => cmd_decode(&hex, symbols),
        Commands::Inspect { text } => cmd_inspect(&text),
    }
}

fn cmd_encode(text: &str) {
    match encode_text(text) {
        Ok(enc) => {
            let bits = enc.symbol_count * 5;
            let sc = size_class(enc.symbol_count);
            println!("text:          {}", text);
            println!("symbols:       {}", enc.symbol_count);
            println!("bits:          {}", bits);
            println!("payload bytes: {}", enc.bytes.len());
            println!("hex:           {}", to_hex(&enc.bytes));
            println!("suitability:   {}", suitability(&sc));
        }
        Err(e) => {
            eprintln!("error: {}", e);
            std::process::exit(1);
        }
    }
}

fn cmd_decode(hex_str: &str, symbol_count: usize) {
    let bytes = match from_hex(hex_str) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("error: invalid hex: {}", e);
            std::process::exit(1);
        }
    };
    match decode_text(&bytes, symbol_count) {
        Ok(text) => println!("{}", text),
        Err(e) => {
            eprintln!("error: {}", e);
            std::process::exit(1);
        }
    }
}

fn cmd_inspect(text: &str) {
    let report = validate_text(text);

    // Count valid symbols regardless of validation result so the report is always useful.
    let symbol_count: usize = text
        .chars()
        .filter(|&c| Symbol::try_from(c).is_ok())
        .count();

    let bits = symbol_count * 5;
    let byte_count = (bits + 7) / 8;
    let sc = size_class(symbol_count);

    println!("text:          {}", text);
    println!("symbols:       {}", symbol_count);
    println!("bits:          {}", bits);
    println!("payload bytes: {}", byte_count);

    // Only show hex when the full text is clean.
    if report.ok {
        if let Ok(enc) = encode_text(text) {
            println!("hex:           {}", to_hex(&enc.bytes));
        }
    }

    println!("suitability:   {}", suitability(&sc));
    println!("valid:         {}", if report.ok { "yes" } else { "no" });

    if !report.issues.is_empty() {
        println!();
        for issue in &report.issues {
            println!("  ! {}", issue.message);
        }
    }
}

fn suitability(sc: &SizeClass) -> &'static str {
    match sc {
        SizeClass::Pulse => "polite lora pulse",
        SizeClass::Bloop => "polite lora bloop",
        SizeClass::LongBloop => "longbloop (pushing lora limits)",
        SizeClass::Flood => "flood (impolite on lora)",
        SizeClass::TooLong => "too long (exceeds flood limit)",
    }
}

fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

fn from_hex(s: &str) -> Result<Vec<u8>, String> {
    let s: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    if s.len() % 2 != 0 {
        return Err("odd number of hex digits".to_string());
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|e| e.to_string()))
        .collect()
}
