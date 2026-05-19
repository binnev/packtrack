use crate::cli::display::{display_job, heading, line};
use crate::cli::settings::Settings;
use clap::Args;
use log;
use packtrack::Result;
use packtrack::api::Job;
use packtrack::api::{Context, track_urls};
use packtrack::cache::FileCache;
use packtrack::url_store::{AnnotatedUrl, FileUrlStore, UrlStore};
use packtrack::utils::check_path_exists;
use std::cmp::Ordering;
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

    /// Don't use the cache (even for completed packages)
    #[arg(short, long)]
    pub no_cache: bool,

    // FIXME: This is only relevant for CLI printout (not JSON)
    /// Display detailed info on completed packages
    #[arg(short, long)]
    pub detail: bool,

    /// Preferred language (passed to the carrier)
    #[arg(short, long)]
    pub language: Option<String>,

    /// Recipient postcode (sometimes required to get full info)
    #[arg(short, long)]
    pub postcode: Option<String>,
}

/// Provide an ordering for two jobs, based on various time fields.
fn order_jobs(a: &Job, b: &Job) -> Ordering {
    let a_package = match &a.result {
        Ok(package) => package,
        Err(_) => return Ordering::Equal,
    };
    let b_package = match &b.result {
        Ok(package) => package,
        Err(_) => return Ordering::Equal,
    };
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
}

/// Display jobs to the user in the CLI
pub fn display_jobs(jobs: Vec<Job>, completed_detail: bool) {
    let mut completed: Vec<Job> = Vec::new(); // Packages with a final status
    let mut in_progress: Vec<Job> = Vec::new();
    let mut errors: Vec<Job> = Vec::new();
    for job in jobs {
        match &job.result {
            Ok(package) => match package.status.is_final() {
                true => completed.push(job),
                false => in_progress.push(job),
            },
            Err(_) => errors.push(job),
        }
    }
    for list in [&mut completed, &mut in_progress] {
        list.sort_by(order_jobs);
    }

    // display final packages
    if completed.len() > 0 {
        heading(&"completed");
        let s = completed
            .iter()
            .map(|job| display_job(job, completed_detail))
            .collect::<Vec<_>>()
            .join("\n");
        println!("{s}")
    }

    if in_progress.len() > 0 {
        heading(&"in progress");
        let line = format!("\n{}\n", line());
        let s = in_progress
            .iter()
            .map(|job| display_job(job, completed_detail))
            .collect::<Vec<_>>()
            .join(&line);
        println!("{s}")
    }

    if errors.len() > 0 {
        // display errors
        heading(&"errors");
        let line = format!("\n{}\n", line());
        let s = errors
            .iter()
            .map(|job| display_job(job, completed_detail))
            .collect::<Vec<_>>()
            .join(&line);
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
    let url_store = FileUrlStore::new(urls_file.clone())?;
    let mut urls = url_store.filter(ctx.filters.url.as_deref());

    // TODO: make this clearer
    if urls.len() == 0 && ctx.filters.url.is_some() {
        urls = vec![AnnotatedUrl::new(
            ctx.filters.url.clone().unwrap(),
            Some("dynamic".into()),
        )]
    }
    let cache_file = settings.cache_file.clone();
    let cache = FileCache::new(cache_file)?;
    let jobs = track_urls(urls, cache, ctx).await?;
    display_jobs(jobs, track_args.detail);
    log::info!("track_all took {:?}", start.elapsed());
    Ok(())
}
