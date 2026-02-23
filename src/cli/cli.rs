use enum_iterator::all;
use packtrack::url_store::AnnotatedUrl;
use packtrack::utils::check_path_exists;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

use crate::cli::display::{display_job, heading, line};
use crate::cli::settings;
use crate::cli::settings::Settings;
use crate::cli::urls;
use clap::Args;
use clap::{Parser, Subcommand};
use log::{self, LevelFilter};
use packtrack::Result;
use packtrack::api::Filters;
use packtrack::api::Job;
use packtrack::api::{Context, track_urls};
use packtrack::tracker::PackageStatus;

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

    let sets = settings::load()?;
    let ctx = Context {
        cache_seconds:      args
            .tracking
            .cache_seconds
            .unwrap_or(sets.cache_seconds.clone()),
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
            .or(sets.postcode.clone()),
        preferred_language: args
            .tracking
            .language
            .clone()
            .or(sets.language.clone())
            .unwrap_or(Context::default().preferred_language),
    };
    log::debug!("Cache seconds: {}", ctx.cache_seconds);

    // Handle subcommands
    match args.subcommand {
        None => track(&sets, &ctx, args.tracking).await?,
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
    let default_file = &settings.urls_file;
    match command {
        UrlCommand::Add {
            url,
            description,
            args,
        } => {
            let file = args
                .urls_file
                .as_ref()
                .unwrap_or(default_file);
            let msg = format!("Added {url}");
            let aurl = AnnotatedUrl::new(url, description);
            match urls::add(file, aurl) {
                Ok(()) => println!("{msg}"),
                Err(err) => return Err(err),
            }
        }
        UrlCommand::Remove { url, args } => {
            let file = args
                .urls_file
                .as_ref()
                .unwrap_or(default_file);
            match urls::remove(file, url) {
                Ok(removed) => {
                    println!("Removed urls:");
                    for url in removed {
                        println!("{url}");
                    }
                }
                Err(err) => return Err(err),
            }
        }
        UrlCommand::List { query, args } => {
            let file = args
                .urls_file
                .as_ref()
                .unwrap_or(default_file);
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

#[derive(Args)]
struct TrackArgs {
    /// Either a new URL, or a fragment of an existing URL
    url: Option<String>,

    /// Path to the URLs file
    #[arg(short, long, value_parser = check_path_exists)]
    urls_file: Option<PathBuf>,

    /// Filter by sender
    #[arg(short, long)]
    sender: Option<String>,

    /// Filter by postal carrier
    #[arg(short, long)]
    carrier: Option<String>,

    /// Filter by recipient
    #[arg(short, long)]
    recipient: Option<String>,

    /// Max age for cache entries to be reused
    #[arg(short = 'C', long)]
    cache_seconds: Option<usize>,

    /// Don't use the cache (even for delivered packages)
    #[arg(short, long)]
    no_cache: bool,

    // FIXME: This is only relevant for CLI printout (not JSON)
    /// Display detailed info on delivered packages
    #[arg(short, long)]
    delivered: bool,

    /// Preferred language (passed to the carrier)
    #[arg(short, long)]
    language: Option<String>,

    /// Recipient postcode (sometimes required to get full info)
    #[arg(short, long)]
    postcode: Option<String>,
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
    List {
        query: Option<String>,
        #[clap(flatten)]
        args:  UrlArgs,
    },
    /// Add a URL to the urls file
    Add {
        url:         String,
        #[arg(short, long)]
        description: Option<String>,
        #[clap(flatten)]
        args:        UrlArgs,
    },
    /// Remove a URL from the urls file
    Remove {
        url:  String,
        #[clap(flatten)]
        args: UrlArgs,
    },
}
#[derive(Args)]
struct UrlArgs {
    /// Path to the URLs file
    #[arg(short, long, value_parser = check_path_exists)]
    urls_file: Option<PathBuf>,
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
    let mut jobs_by_status: HashMap<PackageStatus, Vec<Job>> = HashMap::new();
    for job in jobs {
        match &job.result {
            Ok(package) => {
                let status = package.status();
                jobs_by_status
                    .entry(status)
                    .or_default()
                    .push(job);
            }
            Err(_) => errors.push(job),
        }
    }
    // sort by time
    for (status, packages) in jobs_by_status.iter_mut() {
        if status == &PackageStatus::Delivered {
            packages.sort_by(|a, b| {
                a.result
                    .as_ref()
                    .unwrap() // TODO: make this better
                    .delivered
                    .cmp(&b.result.as_ref().unwrap().delivered)
            });
        }
        if status == &PackageStatus::InTransit {
            packages.sort_by(|a, b| {
                a.result
                    .as_ref()
                    .unwrap()
                    .eta
                    .cmp(&b.result.as_ref().unwrap().eta)
            });
            packages.sort_by(|a, b| {
                let a_time = a.result.as_ref().unwrap().eta.or(a
                    .result
                    .as_ref()
                    .unwrap()
                    .eta_window
                    .as_ref()
                    .map(|w| w.start));
                let b_time = b.result.as_ref().unwrap().eta.or(b
                    .result
                    .as_ref()
                    .unwrap()
                    .eta_window
                    .as_ref()
                    .map(|w| w.start));
                a_time.cmp(&b_time)
            });
        }
    }

    // display successful results
    let line = format!("\n{}\n", line());
    for status in all::<PackageStatus>() {
        let jobs = jobs_by_status
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
        let s = jobs
            .iter()
            .map(|job| display_job(job, delivered_detail))
            .collect::<Vec<_>>()
            .join(&separator);
        println!("{s}");
    }

    // display errors
    heading(&"errors");
    let separator = format!("\n{}\n", "-".repeat(80));
    let s = errors
        .iter()
        .map(|job| display_job(job, delivered_detail))
        .collect::<Vec<_>>()
        .join(&separator);
    println!("{s}");
}

async fn track(
    settings: &Settings,
    ctx: &Context,
    track_args: TrackArgs,
) -> Result<()> {
    let start = Instant::now();
    // TODO: Move this somewhere else, and make it completely stateless, so that
    // you can
    // - Pass no -f arg (use URLs file defined in settings)
    //     - allow filtering by query
    // - Pass -f urls_file (use different URLs file)
    //     - allow filtering by query
    // - Pass one or more URLs as a "\n" separated string
    let urls_file: &PathBuf = track_args
        .urls_file
        .as_ref()
        .unwrap_or(&settings.urls_file);
    let mut urls = urls::filter(urls_file, ctx.filters.url.as_deref())?;

    // TODO: make this clearer
    if urls.len() == 0 && ctx.filters.url.is_some() {
        urls = vec![AnnotatedUrl::new(
            ctx.filters.url.clone().unwrap(),
            Some("dynamic".into()),
        )]
    }
    let jobs = track_urls(urls, ctx).await?;
    display_jobs(jobs, track_args.delivered);
    log::info!("track_all took {:?}", start.elapsed());
    Ok(())
}
