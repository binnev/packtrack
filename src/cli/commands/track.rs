use crate::cli::display::{display_job, heading, line};
use crate::cli::settings::Settings;
use crate::cli::urls;
use clap::Args;
use log;
use packtrack::Result;
use packtrack::api::Job;
use packtrack::api::{Context, track_urls};
use packtrack::tracker::PackageStatus;
use packtrack::url_store::AnnotatedUrl;
use packtrack::utils::check_path_exists;
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
    // THOUGHT: Maybe the sorting by time is enough? InTransit packages will
    // naturally fall below Delivered packages...
    //
    // No. Because there's no guarantee that we can extract any time from a
    // package. It might have no `delivered`, no `eta`, no `eta_window`, and no
    // events to grab a time from either.

    let mut delivered: Vec<Job> = Vec::new();
    let mut delivered_to_neighbour: Vec<Job> = Vec::new();
    let mut in_transit: Vec<Job> = Vec::new();
    let mut errors: Vec<Job> = Vec::new();
    for job in jobs {
        match &job.result {
            Ok(package) => match package.status() {
                PackageStatus::Delivered => delivered.push(job),
                PackageStatus::DeliveredToNeighbour { .. } => {
                    delivered_to_neighbour.push(job)
                }
                PackageStatus::InTransit => in_transit.push(job),
            },
            Err(_) => errors.push(job),
        }
    }
    for list in [&mut delivered, &mut delivered_to_neighbour, &mut in_transit] {
        list.sort_by(|a, b| {
            let a_package = a.result.as_ref().unwrap();
            let b_package = b.result.as_ref().unwrap();
            let a_time = a_package
                .delivered
                .or(a_package.eta)
                .or(a_package
                    .eta_window
                    .as_ref()
                    .map(|w| w.start));
            let b_time = b_package
                .delivered
                .or(b_package.eta)
                .or(b_package
                    .eta_window
                    .as_ref()
                    .map(|w| w.start));
            a_time.cmp(&b_time)
        });
    }

    // display successful results
    let line = format!("\n{}\n", line());
    for jobs in [delivered, delivered_to_neighbour, in_transit] {
        if let Some(first) = jobs.first() {
            let status = first
                .result
                .as_ref()
                .expect("This should be a success!")
                .status();
            let separator = match status {
                PackageStatus::Delivered
                | PackageStatus::DeliveredToNeighbour { .. } => {
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
