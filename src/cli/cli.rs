use clap::Args;
use clap::{Parser, Subcommand, command};
use log::{self, LevelFilter};
use packtrack::api;
use packtrack::api::Context;
use packtrack::api::Filters;
use packtrack::settings;
use packtrack::urls;
use packtrack::{Error, Result};

pub async fn main() -> Result<()> {
    let cli = Cli::parse();

    let verbosity = match cli.globals.verbosity {
        0 => LevelFilter::Off,
        1 => LevelFilter::Error,
        2 => LevelFilter::Warn,
        3 => LevelFilter::Info,
        4 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };
    env_logger::Builder::new()
        .filter(None, verbosity)
        .init();
    log::debug!("Verbosity {verbosity}");

    let ctx = Context {
        cache_seconds:  cli
            .globals
            .cache_seconds
            .unwrap_or(settings::load()?.cache_seconds),
        display_format: None,
        filters:        Filters {
            url:       cli.filter_opts.url,
            sender:    cli.filter_opts.sender,
            recipient: cli.filter_opts.recipient,
            carrier:   cli.filter_opts.carrier,
        },
    };
    log::debug!("Cache seconds: {}", ctx.cache_seconds);

    // Handle subcommands
    match cli.command {
        None => api::track(&ctx).await?,
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
        UrlCommand::List { query } => urls::list(query).await?,
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
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    #[clap(flatten)]
    globals: GlobalArgs,

    #[clap(flatten)]
    filter_opts: FilterOpts,
}

#[derive(Args)]
struct GlobalArgs {
    /// Set verbosity. `-v` = 1, `-vvv` = 3
    #[arg(short, long, action = clap::ArgAction::Count, global=true)]
    verbosity: u8,

    /// Max age for cache entries to be reused
    #[arg(short, long, global = true)]
    cache_seconds: Option<usize>,
}

#[derive(Args)]
struct FilterOpts {
    /// Either a new URL, or a fragment of an existing URL
    url: Option<String>,

    /// Filter by sender
    #[arg(long)]
    sender: Option<String>,

    /// Filter by postal carrier
    #[arg(long)]
    carrier: Option<String>,

    /// Filter by recipient
    #[arg(long)]
    recipient: Option<String>,
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
    List { query: Option<String> },
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
