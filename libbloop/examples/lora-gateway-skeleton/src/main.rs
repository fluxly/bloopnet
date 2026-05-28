mod gateway;

use clap::{Parser, Subcommand};
use gateway::{cmd_listen, cmd_transmit, ListenArgs, TransmitArgs};

#[derive(Parser)]
#[command(
    name = "bloop-lora",
    about = "Bloopnet LoRa gateway skeleton\n\nTransmit and receive Bloop packets over a serial-attached LoRa modem.\nUse --simulate to run without hardware."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Encode a Bloop and transmit it via a LoRa modem
    Transmit {
        /// The text to transmit (must be valid Bloop text)
        text: String,

        /// Serial port connected to the LoRa modem
        #[arg(long, default_value = "/dev/tty.usbserial-0001")]
        port: String,

        /// Serial baud rate
        #[arg(long, default_value_t = 115_200)]
        baud: u32,

        /// Sender ID embedded in the packet header
        #[arg(long, default_value_t = 0)]
        sender_id: u32,

        /// Packet sequence number
        #[arg(long, default_value_t = 1)]
        packet_id: u32,

        /// Simulate transmission without opening a serial port
        #[arg(long)]
        simulate: bool,
    },

    /// Listen on a serial port for incoming Bloop packets
    Listen {
        /// Serial port connected to the LoRa modem
        #[arg(long, default_value = "/dev/tty.usbserial-0001")]
        port: String,

        /// Serial baud rate
        #[arg(long, default_value_t = 115_200)]
        baud: u32,

        /// Simulate reception using a pre-encoded example bloop
        #[arg(long)]
        simulate: bool,
    },
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Commands::Transmit { text, port, baud, sender_id, packet_id, simulate } => {
            cmd_transmit(&TransmitArgs { text, port, baud, sender_id, packet_id, simulate })
        }
        Commands::Listen { port, baud, simulate } => {
            cmd_listen(&ListenArgs { port, baud, simulate })
        }
    };

    if let Err(e) = result {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
