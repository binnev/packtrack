use crate::cli::display::{display_job, heading, line};
use crate::cli::settings::Settings;
use crate::cli::urls;
use clap::Args;
use enum_iterator::all;
use log;
use packtrack::Result;
use packtrack::api::Job;
use packtrack::api::{Context, track_urls};
use packtrack::tracker::PackageStatus;
use packtrack::url_store::AnnotatedUrl;
use packtrack::utils::check_path_exists;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Args)]
pub struct TrackArgs {
    /// Either a new URL, or a fragment of an existing URL
    pub url: Option<String>,

    /// Path to the URLs file
    #[arg(short, long, value_parser = check_path_exists)]
    pub urls_file: Option<PathBuf>,

    /// Filter by sender
    #[arg(short, long)]
    pub sender: Option<String>,

    /// Filter by postal carrier
    #[arg(short, long)]
    pub carrier: Option<String>,

    /// Filter by recipient
    #[arg(short, long)]
    pub recipient: Option<String>,

    /// Max age for cache entries to be reused
    #[arg(short = 'C', long)]
    pub cache_seconds: Option<usize>,

    /// Don't use the cache (even for delivered packages)
    #[arg(short, long)]
    pub no_cache: bool,

    // FIXME: This is only relevant for CLI printout (not JSON)
    /// Display detailed info on delivered packages
    #[arg(short, long)]
    pub delivered: bool,

    /// Preferred language (passed to the carrier)
    #[arg(short, long)]
    pub language: Option<String>,

    /// Recipient postcode (sometimes required to get full info)
    #[arg(short, long)]
    pub postcode: Option<String>,
}

pub fn display_jobs(jobs: Vec<Job>, delivered_detail: bool) {
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
                let a_time = a.result.as_ref().unwrap().delivered;
                let b_time = b.result.as_ref().unwrap().delivered;
                a_time.cmp(&b_time)
            });
        }
        if status == &PackageStatus::InTransit {
            packages.sort_by(|a, b| {
                let a_package = a.result.as_ref().unwrap();
                let a_eta = a_package.eta.or(a_package
                    .eta_window
                    .as_ref()
                    .map(|w| w.start));
                let b_package = b.result.as_ref().unwrap();
                let b_eta = b_package.eta.or(b_package
                    .eta_window
                    .as_ref()
                    .map(|w| w.start));
                a_eta.cmp(&b_eta)
            });
        }
    }

    // display successful results
    let line = format!("\n{}\n", line());
    for status in all::<PackageStatus>() {
        let jobs = jobs_by_status
            .entry(status.clone())
            .or_insert(vec![]);
        if jobs.len() > 0 {
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
    }

    if errors.len() > 0 {
        // display errors
        heading(&"errors");
        let separator = line;
        let s = errors
            .iter()
            .map(|job| display_job(job, delivered_detail))
            .collect::<Vec<_>>()
            .join(&separator);
        println!("{s}");
    }
}

pub async fn track(
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
