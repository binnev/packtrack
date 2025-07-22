use std::fmt::Display;

// use crate::core::tracker::{Package, PackageStatus};
use chrono::{DateTime, Datelike, Local, TimeZone};
use packtrack::tracker::{Event, Package, PackageStatus, TimeWindow};

pub fn heading(s: &dyn Display) {
    println!("{}", "=".repeat(80));
    let text = format!(" {s} ");
    let text = spaced(text).to_uppercase();
    let text = format!("{text:^80}");
    println!("{}", text);
    println!("{}", "=".repeat(80));
}

/// "hello" -> "h e l l o"
pub fn spaced(s: String) -> String {
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

fn display_delivered_package(package: &Package) -> String {
    let time = package
        .delivered
        .map(|dt| display_time(dt))
        .unwrap_or("???".to_owned());
    let mut f = String::new();
    f.push_str(&format!(
        "[{time}] {} Package {}",
        package.channel, package.barcode
    ));
    if let Some(sender) = &package.sender {
        f.push_str(&format!(" from {sender}"));
    }
    if let Some(recipient) = &package.recipient {
        f.push_str(&format!(" to {recipient}"));
    }
    f
}
fn display_in_transit_package(package: &Package) -> String {
    let mut f = String::new();
    f.push_str(&format!("{} Package {}", package.channel, package.barcode));
    if let Some(sender) = package.sender.as_ref() {
        f.push_str(&format!(" from {sender}"));
    } else {
        f.push_str(&format!(""));
    }
    if let Some(eta) = package.eta {
        f.push_str(&format!("expected delivery: {}", display_time(eta)));
    }
    if let Some(window) = package.eta_window.as_ref() {
        f.push_str(&format!("delivery window: {}", display_timewindow(window)));
    }
    f.push_str(&format!("events:"));
    for event in package.events.iter() {
        f.push_str(&format!("\n    {}", display_event(event)));
    }
    f
}
pub fn display_package(package: &Package) -> String {
    match package.status() {
        PackageStatus::Delivered => display_delivered_package(package),
        PackageStatus::InTransit => display_in_transit_package(package),
    }
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
    use packtrack::utils::UtcTime;

    use super::*;
    use crate::Result;

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
}
