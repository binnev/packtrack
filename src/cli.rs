use crate::api;
use crate::settings;
use crate::urls;
use crate::{Error, Result};
use clap::{command, Parser, Subcommand};
use log::{self, LevelFilter};

pub async fn main() -> Result<()> {
    let cli = Cli::parse();

    // TODO: pass this to the logger configuration.
    let log_level = match cli.verbosity {
        0_ => LevelFilter::Off,
        1 => LevelFilter::Error,
        2 => LevelFilter::Warn,
        3 => LevelFilter::Info,
        4 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };
    env_logger::Builder::new()
        .filter(None, log_level)
        .init();

    // Handle subcommands
    match cli.command {
        None => api::main().await?,
        Some(Command::Url { command }) => handle_url_command(command).await?,
        Some(Command::Config { command }) => handle_config_command(command)?,
    }
    Ok(())
}

async fn handle_url_command(command: UrlCommand) -> Result<()> {
    match command {
        UrlCommand::Add { url } => match urls::add(&url).await {
            Ok(()) => println!("Added {url}"),
            Err(Error::Url(err)) => println!("Error: {err}"),
            Err(err) => return Err(err),
        },
        UrlCommand::Remove { url } => match urls::remove(url).await {
            Ok(removed) => {
                println!("Removed urls:");
                for url in removed {
                    println!("{url}");
                }
            }
            Err(Error::Url(err)) => println!("Error: {err}"),
            Err(err) => return Err(err),
        },
        UrlCommand::List => urls::list().await?,
    }
    Ok(())
}

fn handle_config_command(command: ConfigCommand) -> Result<()> {
    match command {
        ConfigCommand::List => settings::print()?,
        ConfigCommand::Set { key, value } => settings::update(key, value)?,
        ConfigCommand::Reset => settings::reset()?,
    }
    Ok(())
}

#[derive(Parser)]
#[command(name = "packtrack")]
#[command(about = "A CLI for tracking packages")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    /// Set verbosity. `-v` = 1, `-vvv` = 3
    #[arg(short, long, action = clap::ArgAction::Count, global=true)]
    verbosity: u8,
}

#[derive(Subcommand)]
enum Command {
    /// URL management
    Url {
        #[command(subcommand)]
        command: UrlCommand,
    },
    /// Configuration
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
}
#[derive(Subcommand)]
enum UrlCommand {
    /// List the URLs currently in the file
    List,
    /// Add a URL to the urls file
    Add { url: String },
    /// Remove a URL from the urls file
    Remove { url: String },
}
#[derive(Subcommand)]
enum ConfigCommand {
    /// List the current settings
    List,
    /// Update the settings
    Set { key: String, value: String },
    /// Reset settings to the defaults
    Reset,
}
