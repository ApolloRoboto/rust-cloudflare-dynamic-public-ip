mod cli;
mod cloudflare;
mod ip_monitor;
mod logger;
mod mqtt;
mod utils;

use logger::LOGGER;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    log::set_logger(&LOGGER).unwrap();
    std::process::exit(cli::run().await);
}
