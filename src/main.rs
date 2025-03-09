mod configuration_api;
mod state;

use clap::Parser;
use std::{net::SocketAddr, path::Path};

mod reverse_proxy;

#[derive(Parser, Debug)]
#[command(name = "Server Config")]
#[command(about = "A simple TLS reverse proxy with dynamic configuration", long_about = None)]
struct Args {
    /// Configuration endpoints address
    #[arg(long)]
    config_addr: SocketAddr,

    /// The address to listen on
    #[arg(long)]
    listen_addr: SocketAddr,

    /// The path to the certificate file
    #[arg(long)]
    cert_path: String,

    /// The path to the certificate key file
    #[arg(long)]
    cert_key_path: String,
}

#[tokio::main]
async fn main() {
    // Parse inputs

    let args = Args::parse();

    // Starts the TLS server and the configuration API

    let configuration_api = configuration_api::start(&args.config_addr);
    let reverse_proxy = reverse_proxy::start(
        &args.listen_addr,
        Path::new(&args.cert_path),
        Path::new(&args.cert_key_path),
    );

    let _ = tokio::try_join!(configuration_api, reverse_proxy);
}
