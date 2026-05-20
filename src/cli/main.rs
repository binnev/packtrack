use crate::cli::cache::{CacheCommand, handle_cache_command};
use crate::cli::config::{ConfigCommand, handle_config_command};
use crate::cli::track::{TrackArgs, track};
use crate::cli::url::{UrlCommand, handle_url_command};
use clap::Args;
use clap::{Parser, Subcommand};
use log::{self, LevelFilter};
use packtrack::Result;
use packtrack::api::Context;
use packtrack::api::Filters;
use packtrack::settings::{FileSettingsManager, get_settings_file};

pub async fn main() -> Result<()> {
    let args = Cli::parse();

    let verbosity = match args.globals.verbosity.as_str() {
        "0" | "off" => LevelFilter::Off,
        "1" | "error" => LevelFilter::Error,
        "2" | "warn" => LevelFilter::Warn,
        "3" | "info" => LevelFilter::Info,
        "4" | "debug" => LevelFilter::Debug,
        "5" | "trace" => LevelFilter::Trace,
        other => return Err(format!("Invalid verbosity: {other}").into()),
    };
    env_logger::Builder::new()
        .filter(None, verbosity)
        .init();
    log::debug!("Verbosity {verbosity}");

    let settings_file = get_settings_file()?;
    let mut settings_manager = FileSettingsManager::new(settings_file)?;
    let settings = &settings_manager.settings;
    let ctx = Context {
        cache_seconds:      args
            .tracking
            .cache_seconds
            .unwrap_or(settings.cache_seconds.clone()),
        use_cache:          !args.tracking.no_cache,
        filters:            Filters {
            url:       args.tracking.url.clone(),
            sender:    args.tracking.sender.clone(),
            recipient: args.tracking.recipient.clone(),
            carrier:   args.tracking.carrier.clone(),
        },
        default_postcode:   args
            .tracking
            .postcode
            .clone()
            .or(settings.postcode.clone()),
        preferred_language: args
            .tracking
            .language
            .clone()
            .or(settings.language.clone())
            .unwrap_or(Context::default().preferred_language),
    };
    log::debug!("Cache seconds: {}", ctx.cache_seconds);

    // Handle subcommands
    match args.subcommand {
        None => track(&settings, &ctx, args.tracking).await?,
        Some(Command::Url { command }) => {
            handle_url_command(command, &settings).await?
        }
        Some(Command::Config { command }) => {
            handle_config_command(command, &mut settings_manager)?
        }
        Some(Command::Cache { command }) => {
            handle_cache_command(command, &settings).await?
        }
    }
    Ok(())
}

#[derive(Parser)]
// `args_conflicts_with_subcommands` makes non-global args only accessible for
// the default subcommand. So all the options related to tracking (sender, etc)
// are not available for the config subcommand, for example.
#[clap(args_conflicts_with_subcommands = true)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    subcommand: Option<Command>,

    #[clap(flatten)]
    tracking: TrackArgs,

    #[clap(flatten)]
    globals: GlobalArgs,
}

#[derive(Args)]
struct GlobalArgs {
    /// Set verbosity
    #[arg(
        short,
        long,
        global = true,
        required = false,
        default_value = "error"
    )]
    verbosity: String,
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
    /// Cache management
    Cache {
        #[command(subcommand)]
        command: CacheCommand,
    },
}
