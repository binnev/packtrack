use enum_iterator::all;
use std::collections::HashMap;
use std::time::Instant;

use crate::cli::settings;
use crate::cli::urls;
use crate::cli::utils::{display_package, heading};
use clap::Args;
use clap::{Parser, Subcommand, command};
use log::{self, LevelFilter};
use packtrack::Result;
use packtrack::api::Filters;
use packtrack::api::Job;
use packtrack::api::{Context, track_urls};
use packtrack::tracker::{Package, PackageStatus};

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

    let sets = settings::load()?;
    let ctx = Context {
        cache_seconds: cli
            .globals
            .cache_seconds
            .unwrap_or(settings::load()?.cache_seconds),
        filters: Filters {
            url:       cli.filter_opts.url,
            sender:    cli.filter_opts.sender,
            recipient: cli.filter_opts.recipient,
            carrier:   cli.filter_opts.carrier,
        },
        default_postcode: sets.postcode,
        ..Default::default() // TODO: get language from settings or CLI flags
    };
    log::debug!("Cache seconds: {}", ctx.cache_seconds);

    // Handle subcommands
    match cli.command {
        None => track(&ctx).await?,
        Some(Command::Url { command }) => handle_url_command(command).await?,
        Some(Command::Config { command }) => handle_config_command(command)?,
    }
    Ok(())
}

async fn handle_url_command(command: UrlCommand) -> Result<()> {
    match command {
        UrlCommand::Add { url } => match urls::add(&url).await {
            Ok(()) => println!("Added {url}"),
            Err(err) => return Err(err),
        },
        UrlCommand::Remove { url } => match urls::remove(url).await {
            Ok(removed) => {
                println!("Removed urls:");
                for url in removed {
                    println!("{url}");
                }
            }
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
    #[arg(short = 'C', long, global = true)]
    cache_seconds: Option<usize>,
}

#[derive(Args)]
struct FilterOpts {
    /// Either a new URL, or a fragment of an existing URL
    url: Option<String>,

    /// Filter by sender
    #[arg(short, long)]
    sender: Option<String>,

    /// Filter by postal carrier
    #[arg(short, long)]
    carrier: Option<String>,

    /// Filter by recipient
    #[arg(short, long)]
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

fn display_jobs(jobs: Vec<Job>) {
    // sort the results by status / error
    let mut errors: Vec<Job> = vec![];
    let mut jobs_by_status: HashMap<PackageStatus, Vec<Package>> =
        HashMap::new();
    for job in jobs {
        match &job.result {
            Ok(package) => {
                let status = package.status();
                jobs_by_status
                    .entry(status)
                    .and_modify(|e| e.push(package.clone()))
                    .or_insert(vec![package.clone()]);
            }
            Err(_) => errors.push(job),
        }
    }
    // sort by time
    for (status, packages) in jobs_by_status.iter_mut() {
        if status == &PackageStatus::Delivered {
            packages.sort_by(|a, b| a.delivered.cmp(&b.delivered));
        }
        if status == &PackageStatus::InTransit {
            packages.sort_by(|a, b| a.eta.cmp(&b.eta));
            packages.sort_by(|a, b| {
                let a_time = a
                    .eta
                    .or(a.eta_window.as_ref().map(|w| w.start));
                let b_time = b
                    .eta
                    .or(b.eta_window.as_ref().map(|w| w.start));
                a_time.cmp(&b_time)
            });
        }
    }

    // display successful results
    for status in all::<PackageStatus>() {
        let packages = jobs_by_status
            .entry(status.clone())
            .or_insert(vec![]);
        let separator = match status {
            PackageStatus::Delivered => "\n".to_owned(),
            PackageStatus::InTransit => {
                format!("\n{}\n", "-".repeat(80))
            }
        };
        heading(&status);
        let s = packages
            .iter()
            .map(|package| display_package(package))
            .collect::<Vec<_>>()
            .join(&separator);
        println!("{s}");
    }

    // display errors
    heading(&"errors");
    let separator = format!("\n{}\n", "-".repeat(80));
    let s = errors
        .iter()
        .map(|job| format!("{}\n{:?}", job.url, job.result))
        .collect::<Vec<_>>()
        .join(&separator);
    println!("{s}");
}

async fn track(ctx: &Context) -> Result<()> {
    let start = Instant::now();
    let mut urls = urls::filter(ctx.filters.url.as_deref())?;
    if urls.len() == 0 && ctx.filters.url.is_some() {
        urls = vec![ctx.filters.url.clone().unwrap()]
    }
    let jobs = track_urls(urls, ctx).await?;
    display_jobs(jobs);
    log::info!("track_all took {:?}", start.elapsed());
    Ok(())
}
