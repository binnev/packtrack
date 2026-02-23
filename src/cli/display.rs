use std::fmt::Display;

// use crate::core::tracker::{Package, PackageStatus};
use chrono::{DateTime, Datelike, Local, TimeZone};
use packtrack::{
    api::Job,
    tracker::{Event, Package, PackageStatus, TimeWindow},
};

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

fn display_package_oneliner(package: &Package) -> String {
    let time = package
        .delivered
        .map(|dt| display_time(dt))
        .unwrap_or("???".to_owned());
    let mut f = String::new();
    f.push_str(&format!("[{time}] {} {}", package.channel, package.barcode));
    if let Some(sender) = &package.sender {
        f.push_str(&format!(" from {sender}"));
    }
    if let Some(recipient) = &package.recipient {
        f.push_str(&format!(" to {recipient}"));
    }
    f
}
fn display_package_full(package: &Package) -> String {
    let mut f = String::new();
    f.push_str(&format!("{} Package {}", package.channel, package.barcode));
    if let Some(sender) = package.sender.as_ref() {
        f.push_str(&format!(" from {sender}"));
    } else {
        f.push_str(&format!(""));
    }
    if let Some(recipient) = package.recipient.as_ref() {
        f.push_str(&format!(" to {recipient}"));
    }
    if let Some(eta) = package.eta {
        f.push_str(&format!("\nexpected delivery: {}", display_time(eta)));
    }
    if let Some(window) = package.eta_window.as_ref() {
        f.push_str(&format!(
            "\ndelivery window: {}",
            display_timewindow(window)
        ));
    }
    f.push_str(&format!("\nevents:"));
    for event in package.events.iter() {
        f.push_str(&format!("\n    {}", display_event(event)));
    }
    f
}
pub fn display_package(
    package: &Package,
    // if true, display delivered packages in full detail
    delivered_detail: bool,
) -> String {
    match package.status() {
        PackageStatus::Delivered if !delivered_detail => {
            display_package_oneliner(package)
        }
        _ => display_package_full(package),
    }
}
pub fn display_job(job: &Job, delivered_detail: bool) -> String {
    match &job.result {
        Ok(package) => match package.status() {
            PackageStatus::Delivered if !delivered_detail => {
                display_job_oneliner(job, delivered_detail)
            }
            _ => display_job_full(job, delivered_detail),
        },
        Err(_) => display_job_error(job),
    }
}
fn display_job_oneliner(job: &Job, delivered_detail: bool) -> String {
    let mut s = String::new();
    s += &display_package(job.result.as_ref().unwrap(), delivered_detail);
    if let Some(description) = &job.url.description {
        s += &format!(" ({})", description);
    }
    return s;
}
fn display_job_full(job: &Job, delivered_detail: bool) -> String {
    let mut s = String::new();
    if let Some(description) = &job.url.description {
        s += &format!("Description: {}\n", description);
    }
    s += &display_package(job.result.as_ref().unwrap(), delivered_detail);
    return s;
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
