use crate::cli::Cli;
use crate::http_client::{client, handle_request};
use crate::metrics::register_metrics;
use clap::Parser;
use log::{debug, error, info};
use std::net::SocketAddr;
use std::time::Instant;

mod api;
mod cli;
mod http_client;
mod metrics;


#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    pretty_env_logger::init();

    // TODO: This leaks the API key to stdout. Is this acceptable?
    info!("Jellyfin Exporter v{} starting...", env!("CARGO_PKG_VERSION"));
    debug!("Using options {}", cli);

    let client = client(&cli);
    let mut metrics = register_metrics();
    let exporter = prometheus_exporter::start(SocketAddr::new(cli.jellyfin_exporter_address, cli.jellyfin_exporter_port)).expect("Failed to start the exporter!");

    loop {
        let _guard = exporter.wait_request();
        let s = Instant::now();
        if let Err(err) = handle_request(&cli, &client, &mut metrics).await {
            error!("Failed to handle request: {:?}", err)
        }

        debug!("Done handling request in {:?}", Instant::now() - s)
    }
}
