use crate::cli::display::{display_job, heading, line};
use clap::Args;
use log;
use packtrack::Result;
use packtrack::api::Job;
use packtrack::api::{Context, track_urls};
use packtrack::cache::FileCache;
use packtrack::settings::Settings;
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
        let separator = if completed_detail {
            format!("\n{}\n", line())
        } else {
            "\n".into()
        };
        let s = completed
            .iter()
            .map(|job| display_job(job, completed_detail))
            .collect::<Vec<_>>()
            .join(&separator);
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

#[cfg(test)]
mod tests {
    use packtrack::tracker::{Event, Package, PackageStatus, TimeWindow};

    use super::*;
    fn get_jobs() -> Result<Vec<Job>> {
        Ok(vec![
            Job {
                url:    "https://www.dhl.com/nl-nl/home/tracking.html?submit=1&tracking-id=DHL1"
                    .into(),
                result: Ok(Package {
                    barcode:    "DHL1".into(),
                    channel:    "DHL".into(),
                    sender:     Some("Bol.com".into()),
                    recipient:  Some("Packtrack user".into()),
                    status:     PackageStatus::DeliveredToNeighbour {
                        address: "Streetname 420".into(),
                    },
                    delivered:  Some("2026-06-18T11:30:00Z".parse()?),
                    eta:        Some("2026-06-18T12:00:00Z".parse()?),
                    eta_window: Some(TimeWindow {
                        start: "2026-06-18T10:00:00Z".parse()?,
                        end:   "2026-06-18T14:00:00Z".parse()?,
                    }),
                    events:     vec![
                        Event {
                            text:      "Package accepted".into(),
                            timestamp: "2026-06-16T12:00:00Z".parse()?,
                        },
                        Event {
                            text:      "Package sorted at depot".into(),
                            timestamp: "2026-06-17T12:00:00Z".parse()?,
                        },
                        Event {
                            text:      "Package out for delivery".into(),
                            timestamp: "2026-06-18T12:00:00Z".parse()?,
                        },
                        Event {
                            text:      "Package delivered to neighbour".into(),
                            timestamp: "2026-06-18T13:00:00Z".parse()?,
                        },
                    ],
                }),
            },
            Job {
                url:
                AnnotatedUrl{
                    url: "https://jouw.postnl.nl/track-and-trace/POSTNL1-NL-1234AB".into(),
                    description: Some("shoes".into()), 
                    created: None,

                },
                result: Ok(Package {
                    barcode:    "POSTNL1".into(),
                    sender:     Some("Zalando".into()),
                    recipient:  Some("Packtrack user".into()),
                    status:     PackageStatus::Delivered,
                    channel:    "PostNL".into(),
                    delivered:  Some("2026-06-18T12:00:00Z".parse()?),
                    eta:        Some("2026-06-18T12:00:00Z".parse()?),
                    eta_window: Some(TimeWindow {
                        start: "2026-06-18T10:00:00Z".parse()?,
                        end:   "2026-06-18T14:00:00Z".parse()?,
                    }),
                    events:     vec![
                        Event {
                            text:      "Package accepted".into(),
                            timestamp: "2026-06-16T12:00:00Z".parse()?,
                        },
                        Event {
                            text:      "Package sorted at depot".into(),
                            timestamp: "2026-06-17T12:00:00Z".parse()?,
                        },
                        Event {
                            text:      "Package out for delivery".into(),
                            timestamp: "2026-06-18T12:00:00Z".parse()?,
                        },
                        Event {
                            text:      "Package delivered".into(),
                            timestamp: "2026-06-18T13:00:00Z".parse()?,
                        },
                    ],
                }),
            },
            Job {
                url:
                    "https://jouw.postnl.nl/track-and-trace/POSTNL2-NL-1234AB"
                        .into(),
                result: Ok(Package {
                    channel:    "PostNL".into(),
                    barcode:    "POSTNL2".into(),
                    sender:     Some("Packtrack user".into()),
                    recipient:  Some("Zalando".into()),
                    status:     PackageStatus::InTransit,
                    delivered:  None,
                    eta:        Some("2026-06-18T12:00:00Z".parse()?),
                    eta_window: Some(TimeWindow {
                        start: "2026-06-18T10:00:00Z".parse()?,
                        end:   "2026-06-18T14:00:00Z".parse()?,
                    }),
                    events:     vec![
                        Event {
                            text:      "Package accepted".into(),
                            timestamp: "2026-06-16T12:00:00Z".parse()?,
                        },
                        Event {
                            text:      "Package sorted at depot".into(),
                            timestamp: "2026-06-17T12:00:00Z".parse()?,
                        },
                        Event {
                            text:      "Package out for delivery".into(),
                            timestamp: "2026-06-18T12:00:00Z".parse()?,
                        },
                    ],
                }),
            },
            Job {
                url: "https://www.dhl.com/nl-nl/home/tracking.html?submit=1&tracking-id=DHL2".into(),
                result: Ok(Package {
                    channel:    "DHL".into(),
                    barcode:    "DHL2".into(),
                    sender:     Some("Packtrack user".into()),
                    recipient:  Some("Bol.com".into()),
                    status:     PackageStatus::InTransit,
                    delivered:  None,
                    eta:        Some("2026-06-18T12:00:00Z".parse()?),
                    eta_window: Some(TimeWindow {
                        start: "2026-06-18T10:00:00Z".parse()?,
                        end:   "2026-06-18T14:00:00Z".parse()?,
                    }),
                    events:     vec![
                        Event {
                            text:      "Package accepted".into(),
                            timestamp: "2026-06-16T12:00:00Z".parse()?,
                        },
                        Event {
                            text:      "Package sorted at depot".into(),
                            timestamp: "2026-06-17T12:00:00Z".parse()?,
                        },
                        Event {
                            text:      "Package out for delivery".into(),
                            timestamp: "2026-06-18T12:00:00Z".parse()?,
                        },
                    ],
                }),
            },
        ])
    }

    #[test]
    fn test_display_jobs_all() -> Result<()> {
        let jobs = get_jobs()?;
        println!("## Track all URLs");
        println!(
            "To track all the URLs in your URLs file and receive a summary, simply run packtrack with no arguments: "
        );
        println!("```");
        println!("❯ packtrack");
        display_jobs(jobs, false);
        println!("```");

        println!("## Track a specific URL");
        println!(
            "You can also filter for URLs that contain a given string. The package's barcode or tracking code often works here, because it is usually in the URL."
        );
        println!("```");
        println!("❯ packtrack DHL1");
        let mut jobs = get_jobs()?;
        jobs = jobs
            .into_iter()
            .filter(|j| j.result.as_ref().unwrap().barcode == "DHL1")
            .collect();
        display_jobs(jobs, false);
        println!("```");
        println!(
            "You can also pass a whole new URL. If packtrack can't find the string in your URLs file, it will assume it is a new URL and track it"
        );

        println!("## Filter by carrier");
        println!("Filter for packages carried by PostNL:");
        println!("```");
        println!("❯ packtrack --carrier postnl");
        let mut jobs = get_jobs()?;
        jobs = jobs
            .into_iter()
            .filter(|j| j.result.as_ref().unwrap().channel == "PostNL")
            .collect();
        display_jobs(jobs, false);
        println!("```");

        println!("## Filter by sender");
        println!("Filter for packages sent by Zalando:");
        println!("```");
        println!("❯ packtrack --sender zalando");
        let mut jobs = get_jobs()?;
        jobs = jobs
            .into_iter()
            .filter(|j| {
                j.result.as_ref().unwrap().sender == Some("Zalando".into())
            })
            .collect();
        display_jobs(jobs, false);
        println!("```");

        println!("## Filter by recipient");
        println!("Filter for packages sent _to_ Zalando:");
        println!("```");
        println!("❯ packtrack --recipient zalando");
        let mut jobs = get_jobs()?;
        jobs = jobs
            .into_iter()
            .filter(|j| {
                j.result.as_ref().unwrap().recipient == Some("Zalando".into())
            })
            .collect();
        display_jobs(jobs, false);
        println!("```");
        Ok(())
    }
}
