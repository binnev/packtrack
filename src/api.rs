use crate::error::{Error, Result};
use crate::tracker::get_handler;
use crate::tracker::{Package, PackageStatus};
use crate::urls;
use crate::{settings, tracker};
use chrono::{DateTime, Local, TimeZone};
use enum_iterator::all;
use log;
use std::collections::HashMap;
use std::fmt::Display;
use std::iter::repeat;
use std::path::Path;
use std::time::Instant;
use std::{env, fs};

// -- Public API
pub async fn track_all() -> Result<()> {
    let start = Instant::now();
    let urls = urls::load()?;
    track_urls(urls).await?;
    log::info!("Operation took {:?}", start.elapsed());
    Ok(())
}

pub async fn track(url: &str) -> Result<()> {
    let package = track_url(url).await?;
    println!("{} Package {}", package.channel, package.barcode);
    if let Some(sender) = package.sender.as_ref() {
        println!("\tfrom {sender}");
    }
    if let Some(recipient) = package.recipient.as_ref() {
        println!("\tto {recipient}");
    }
    if let Some(eta) = package.eta {
        println!("\texpected delivery: {}", display_time(eta));
    }
    if let Some(window) = package.eta_window.as_ref() {
        println!("\tdelivery window: {window}");
    }
    if let Some(time) = package.delivered {
        println!("\tdelivered at {time}");
    }
    println!("\tevents:");
    for event in package.events.iter() {
        print!("\t\t{event}");
    }
    Ok(())
}
// -- Internals

/// Get the Tracker implementation for the given URL, and track the package.
async fn track_url(url: &str) -> Result<Package> {
    let tracker = get_handler(url)?;
    tracker.track(url).await
}

/// Track all the URLs in the URLs file.
async fn track_urls(urls: Vec<String>) -> Result<()> {
    // fire off all the tasks in parallel
    let tasks: Vec<_> = urls
        .iter()
        .map(|url| track_url(url))
        .collect();
    let results = futures::future::join_all(tasks).await;

    // sort the results by status / error
    let mut errors: Vec<Error> = vec![];
    let mut packages_by_status: HashMap<PackageStatus, Vec<Package>> =
        HashMap::new();
    for result in results {
        match result {
            Ok(package) => {
                let status = package.status();
                packages_by_status
                    .entry(status)
                    .and_modify(|e| e.push(package.clone()))
                    .or_insert(vec![package]);
            }
            Err(err) => errors.push(err),
        }
    }
    // sort by time
    for (status, packages) in packages_by_status.iter_mut() {
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
        let mut packages = packages_by_status
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
            .map(|package| format!("{package}"))
            .collect::<Vec<_>>()
            .join(&separator);
        println!("{s}");
    }

    // display errors
    heading(&"errors");
    let separator = format!("\n{}\n", "-".repeat(80));
    let s = errors
        .iter()
        .map(|err| format!("{err:?}"))
        .collect::<Vec<_>>()
        .join(&separator);
    println!("{s}");

    Ok(())
}

fn heading(s: &dyn Display) {
    println!("{}", "=".repeat(80));
    let text = format!(" {s} ");
    let text = spaced(text).to_uppercase();
    let text = format!("{text:^80}");
    println!("{}", text);
    println!("{}", "=".repeat(80));
}

/// "hello" -> "h e l l o"
fn spaced(s: String) -> String {
    s.chars()
        .map(|c| c.to_string())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Display the date as "Fri 22 Nov" or "Today"
pub fn display_date<T: TimeZone>(dt: DateTime<T>) -> String {
    let local = dt.with_timezone(&Local);
    let is_today = local.date_naive() == Local::now().date_naive();
    if is_today {
        "Today".into()
    } else {
        local.format("%a %d %b").to_string()
    }
}

/// Display a datetime as "Fri 22 Nov 12:00"
pub fn display_time<T: TimeZone>(dt: DateTime<T>) -> String {
    let local = dt.with_timezone(&Local);
    format!("{} {}", display_date(dt), local.format("%H:%M"))
}
