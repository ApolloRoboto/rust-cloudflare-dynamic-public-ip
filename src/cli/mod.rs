use clap::{Parser, Subcommand};
mod commands;

#[derive(Debug, Parser)]
#[command(name = "Cloudflare Dynamic Public IP", bin_name = "cfdpip", author="ApolloRoboto", version, about="Automatically update public ip address in cloudflare dns records", long_about = None)]
pub struct Cli {
    #[arg(short, long, value_enum, default_value_t = LevelFilterArgument::Info, help = "Set verbosity level")]
    pub verbose: LevelFilterArgument,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(clap::ValueEnum, Debug, Clone)]
pub enum LevelFilterArgument {
    Off,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl LevelFilterArgument {
    pub fn level_filter(&self) -> log::LevelFilter {
        match self {
            LevelFilterArgument::Off => log::LevelFilter::Off,
            LevelFilterArgument::Error => log::LevelFilter::Error,
            LevelFilterArgument::Warn => log::LevelFilter::Warn,
            LevelFilterArgument::Info => log::LevelFilter::Info,
            LevelFilterArgument::Debug => log::LevelFilter::Debug,
            LevelFilterArgument::Trace => log::LevelFilter::Trace,
        }
    }
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[command(about = "Print the current public IP")]
    Current(commands::CurrentArguments),
    #[command(
        about = "Print the affected DNS records, useful to test the connection to Cloudflare"
    )]
    Check(commands::InfoArguments),
    #[command(about = "Monitor and update DNS records on Cloudflare when the public IP changes")]
    Monitor(commands::MonitorArguments),
}

impl Commands {
    pub async fn run(&self) -> i32 {
        match self {
            Commands::Monitor(args) => commands::monitor_command(args).await,
            Commands::Check(args) => commands::check_command(args).await,
            Commands::Current(args) => commands::current_command(args).await,
        }
    }
}

pub async fn run() -> i32 {
    let parsed_cli = Cli::parse();

    log::set_max_level(parsed_cli.verbose.level_filter());

    parsed_cli.command.run().await
}
