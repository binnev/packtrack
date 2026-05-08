/// Functions to display stuff in the CLI
use std::fmt::Display;

use byte_unit::{Byte, UnitType};
use chrono::{DateTime, Datelike, Local, TimeZone, format};
use packtrack::{
    api::Job,
    tracker::{Event, Package, PackageStatus, TimeWindow},
};

/// Print a nice heading with the given text like this:
/// ╭──────────────────────────────────────────────────────────────────────────╮
/// │                D E L I V E R E D   T O   N E I G H B O U R               │
/// ╰──────────────────────────────────────────────────────────────────────────╯
pub fn heading(s: &dyn Display) {
    println!("╭{}╮", "─".repeat(78));
    let text = format!(" {s} ");
    let text = spaced(&text).to_uppercase();
    println!("│{text:^78}│");
    println!("╰{}╯", "─".repeat(78));
}

pub fn line() -> String {
    return "─".repeat(80);
}

/// "hello" -> "h e l l o"
/// InTransit -> I N   T R A N S I T
pub fn spaced(s: &str) -> String {
    let mut out = String::new();
    for char in s.chars() {
        if out.len() > 0 && char.is_uppercase() {
            out.push(' ');
        }
        out.push(char);
    }
    out.chars()
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

pub fn display_timewindow(tw: &TimeWindow) -> String {
    let start = tw.start.with_timezone(&Local);
    let end = tw.end.with_timezone(&Local);
    if start.day() == end.day() {
        format!(
            "{} {} -- {}",
            display_date(start),
            start.format("%H:%M"),
            end.format("%H:%M"),
        )
    } else {
        format!("{} -- {}", display_time(start), display_time(end))
    }
}

pub fn display_event(event: &Event) -> String {
    format!("[{}] {}", display_time(event.timestamp), event.text)
}

pub fn display_job(job: &Job, completed_detail: bool) -> String {
    match &job.result {
        Ok(package) => match package.status.is_final() {
            true if !completed_detail => display_job_oneliner(job, package),
            _ => display_job_full(job, package),
        },
        Err(_) => display_job_error(job),
    }
}

fn display_job_oneliner(job: &Job, package: &Package) -> String {
    let mut parts: Vec<String> = Vec::new();
    // FIXME: This will become a problem when we introduce a new final status
    // for packages that have reached the international border. They are final
    // because they'll receive no more updates, but they're not delivered. We
    // should probably attempt to use the timestamp from the last update.
    let time = package
        .delivered
        .map(|dt| display_time(dt))
        .unwrap_or("????????????????".to_owned());

    parts.push(format!("[{time}] {} {}", package.channel, package.barcode));
    if let Some(sender) = &package.sender {
        parts.push(format!("from {sender}"));
    }
    if let Some(recipient) = &package.recipient {
        parts.push(format!("to {recipient}"));
    }
    if let Some(description) = &job.url.description {
        parts.push(format!("({description})"));
    }
    let mut out = parts.join(" ");
    match &package.status {
        PackageStatus::DeliveredToNeighbour { .. } => {
            out += &format!("\n  ╰─ {}", display_status(&package.status));
        }
        _ => {}
    }

    out
}

fn display_job_full(job: &Job, package: &Package) -> String {
    let mut parts: Vec<String> = Vec::new();
    parts.push(format!("{} {}", package.channel, package.barcode));
    if let Some(description) = &job.url.description {
        parts.push(format!("Description: {}", description));
    }
    parts.push(format!("URL: {}", job.url.url));
    parts.push(format!("Status: {}", display_status(&package.status)));
    if let Some(sender) = package.sender.as_ref() {
        parts.push(format!("From: {sender}"));
    }
    if let Some(recipient) = package.recipient.as_ref() {
        parts.push(format!("To: {recipient}"));
    }
    if let Some(eta) = package.eta {
        parts.push(format!("ETA: {}", display_time(eta)));
    }
    if let Some(window) = package.eta_window.as_ref() {
        parts.push(format!("ETA window: {}", display_timewindow(window)));
    }
    parts.push(format!("events:"));
    for event in package.events.iter() {
        parts.push(format!("    {}", display_event(event)));
    }

    return parts.join("\n");
}

fn display_job_error(job: &Job) -> String {
    let mut parts: Vec<String> = vec![];
    if let Some(description) = &job.url.description {
        parts.push(format!("Description: {description}"))
    }
    parts.push(format!("URL: {}", job.url.url.clone()));
    parts.push(format!("Error: {}", job.result.as_ref().err().unwrap()));
    return parts.join("\n");
}

pub fn human_readable_bytes(bytes: u64) -> String {
    let human_readable =
        Byte::from_u64(bytes).get_appropriate_unit(UnitType::Binary);
    format!("{human_readable:#.1}")
}

fn display_status(s: &PackageStatus) -> String {
    use PackageStatus::*;
    match s {
        Delivered => "Delivered".into(),
        DeliveredToNeighbour { address } => {
            format!("Delivered to neighbour at {address}")
        }
        InTransit => "In transit".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use packtrack::Result;
    use packtrack::utils::UtcTime;

    #[test]
    fn test_display_time() -> Result<()> {
        let utc_time: UtcTime = "2024-11-19T12:00:00+00:00".parse()?;
        assert_eq!(display_time(utc_time), "Tue 19 Nov 13:00");
        Ok(())
    }

    #[test]
    fn test_timewindow_display() -> Result<()> {
        let window = TimeWindow {
            start: "2024-11-19T12:00:00Z".parse()?,
            end:   "2024-11-19T13:00:00Z".parse()?,
        };
        assert_eq!(display_timewindow(&window), "Tue 19 Nov 13:00 -- 14:00");

        let window = TimeWindow {
            start: "2024-11-19T12:00:00Z".parse()?,
            end:   "2024-11-20T13:00:00Z".parse()?,
        };
        assert_eq!(
            display_timewindow(&window),
            "Tue 19 Nov 13:00 -- Wed 20 Nov 14:00"
        );
        Ok(())
    }

    #[test]
    fn test_spaced() {
        assert_eq!(spaced("hello"), "h e l l o",);
        assert_eq!(spaced("contains space"), "c o n t a i n s   s p a c e",);
        assert_eq!(spaced("Delivered"), "D e l i v e r e d",);
        assert_eq!(spaced("InTransit"), "I n   T r a n s i t",)
    }
}
