use enum_iterator::all;
use std::collections::HashMap;
use std::time::Instant;

use crate::cli::settings;
use crate::cli::settings::Settings;
use crate::cli::urls;
use crate::cli::utils::{display_package, heading};
use clap::Args;
use clap::{Parser, Subcommand};
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
        cache_seconds:      cli
            .globals
            .cache_seconds
            .unwrap_or(sets.cache_seconds.clone()),
        use_cache:          !cli.globals.no_cache,
        filters:            Filters {
            url:       cli.filter_opts.url,
            sender:    cli.filter_opts.sender,
            recipient: cli.filter_opts.recipient,
            carrier:   cli.filter_opts.carrier,
        },
        default_postcode:   cli
            .globals
            .postcode
            .or(sets.postcode.clone()),
        preferred_language: cli
            .globals
            .language
            .or(sets.language.clone())
            .unwrap_or(Context::default().preferred_language),
    };
    log::debug!("Cache seconds: {}", ctx.cache_seconds);

    // Handle subcommands
    match cli.command {
        None => track(&sets, &ctx, cli.globals.delivered).await?,
        Some(Command::Url { command }) => {
            handle_url_command(command, &sets).await?
        }
        Some(Command::Config { command }) => {
            handle_config_command(command, sets)?
        }
    }
    Ok(())
}

/// URL file management
async fn handle_url_command(
    command: UrlCommand,
    settings: &Settings,
) -> Result<()> {
    let file = &settings.urls_file;
    match command {
        UrlCommand::Add { url } => match urls::add(file, &url) {
            Ok(()) => println!("Added {url}"),
            Err(err) => return Err(err),
        },
        UrlCommand::Remove { url } => match urls::remove(file, url) {
            Ok(removed) => {
                println!("Removed urls:");
                for url in removed {
                    println!("{url}");
                }
            }
            Err(err) => return Err(err),
        },
        UrlCommand::List { query } => {
            let urls = urls::filter(file, query.as_deref())?;
            for url in urls {
                println!("{url}");
            }
        }
    }
    Ok(())
}

fn handle_config_command(command: ConfigCommand, sets: Settings) -> Result<()> {
    match command {
        ConfigCommand::List => settings::print()?,
        ConfigCommand::Set { key, value } => {
            let sets = sets.update(&key, value)?;
            settings::save(&sets)?;
        }
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

    /// Don't use the cache (even for delivered packages)
    #[arg(short, long, global = true)]
    no_cache: bool,

    // FIXME: This is only relevant for CLI printout (not JSON)
    /// Display detailed info on delivered packages
    #[arg(short, long, global = true)]
    delivered: bool,

    /// Preferred language (passed to the carrier)
    #[arg(short, long, global = true)]
    language: Option<String>,

    /// Recipient postcode (sometimes required to get full info)
    #[arg(short, long, global = true)]
    postcode: Option<String>,
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

fn display_jobs(jobs: Vec<Job>, delivered_detail: bool) {
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
    let line = format!("\n{}\n", "-".repeat(80));
    for status in all::<PackageStatus>() {
        let packages = jobs_by_status
            .entry(status.clone())
            .or_insert(vec![]);
        let separator = match status {
            PackageStatus::Delivered => {
                if delivered_detail {
                    line.clone()
                } else {
                    "\n".to_owned()
                }
            }
            PackageStatus::InTransit => line.clone(),
        };
        heading(&status);
        let s = packages
            .iter()
            .map(|package| display_package(package, delivered_detail))
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

async fn track(
    settings: &Settings,
    ctx: &Context,
    delivered_detail: bool,
) -> Result<()> {
    let start = Instant::now();
    // TODO: Move this somewhere else, and make it completely stateless, so that
    // you can
    // - Pass no -f arg (use URLs file defined in settings)
    //     - allow filtering by query
    // - Pass -f urls_file (use different URLs file)
    //     - allow filtering by query
    // - Pass one or more URLs as a "\n" separated string
    let mut urls =
        urls::filter(&settings.urls_file, ctx.filters.url.as_deref())?;
    if urls.len() == 0 && ctx.filters.url.is_some() {
        urls = vec![ctx.filters.url.clone().unwrap()]
    }
    let jobs = track_urls(urls, ctx).await?;
    display_jobs(jobs, delivered_detail);
    log::info!("track_all took {:?}", start.elapsed());
    Ok(())
}
