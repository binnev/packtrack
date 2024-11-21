use chrono_tz::{Europe::Amsterdam, Tz};
use std::fmt::Display;

use chrono::{DateTime, Datelike, Local, TimeZone, Utc};
use enum_iterator::Sequence;

#[derive(Debug, Clone)]
pub struct Package {
    pub url:        String,
    pub barcode:    String,
    pub channel:    String,
    pub sender:     Option<String>,
    pub recipient:  Option<String>,
    pub eta:        Option<UtcTime>,
    pub eta_window: Option<TimeWindow>,
    pub delivered:  Option<UtcTime>,
    pub events:     Vec<Event>,
}
impl Package {
    pub fn status(&self) -> PackageStatus {
        match self.delivered {
            Some(time) => PackageStatus::Delivered,
            None => PackageStatus::InTransit,
        }
    }
    fn display_delivered(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        let time = self
            .delivered
            .map(|dt| display_time(dt))
            .unwrap_or("???".to_owned());
        write!(f, "[{time}] {} Package {}", self.channel, self.barcode)?;
        if let Some(sender) = &self.sender {
            write!(f, " from {sender}")?;
        }
        if let Some(recipient) = &self.recipient {
            write!(f, " to {recipient}")?;
        }
        Ok(())
    }
    fn display_in_transit(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        write!(f, "{} Package {}", self.channel, self.barcode)?;
        if let Some(sender) = self.sender.as_ref() {
            writeln!(f, " from {sender}")?;
        } else {
            writeln!(f, "");
        }
        if let Some(eta) = self.eta {
            writeln!(f, "expected delivery: {}", display_time(eta))?;
        }
        if let Some(window) = self.eta_window.as_ref() {
            writeln!(f, "delivery window: {window}")?;
        }
        writeln!(f, "events:")?;
        for event in self.events.iter() {
            write!(f, "    {event}");
        }
        Ok(())
    }
}
impl Display for Package {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.status() {
            PackageStatus::Delivered => self.display_delivered(f),
            PackageStatus::InTransit => self.display_in_transit(f),
        }
    }
}
fn display_date<T: TimeZone>(dt: DateTime<T>) -> String {
    let local = dt.with_timezone(&Local);
    let is_today = local.date_naive() == Local::now().date_naive();
    if is_today {
        "Today".into()
    } else {
        local.format("%a %d %b").to_string()
    }
}
fn display_time<T: TimeZone>(dt: DateTime<T>) -> String {
    let local = dt.with_timezone(&Local);
    format!("{} {}", display_date(dt), local.format("%H:%M"))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeWindow {
    pub start: UtcTime,
    pub end:   UtcTime,
}
impl Display for TimeWindow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let start = self.start.with_timezone(&Local);
        let end = self.end.with_timezone(&Local);
        if start.day() == end.day() {
            write!(
                f,
                "{} {} -- {}",
                display_date(start),
                start.format("%H:%M"),
                end.format("%H:%M"),
            )?;
        } else {
            write!(f, "{} -- {}", display_time(start), display_time(end))?;
        }
        Ok(())
    }
}
#[derive(Debug, Clone)]
pub struct Event {
    pub timestamp: UtcTime,
    pub text:      String,
}
impl Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "[{}] {}", display_time(self.timestamp), self.text)
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Sequence, Clone)]
pub enum PackageStatus {
    Delivered,
    InTransit,
}
impl Display for PackageStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // just using debug for now
        write!(f, "{self:?}")
    }
}

pub type UtcTime = DateTime<Utc>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Result;
    use chrono::TimeZone;
    use chrono_tz::Europe::London;

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
        assert_eq!(format!("{window}"), "Tue 19 Nov 13:00 -- 14:00");

        let window = TimeWindow {
            start: "2024-11-19T12:00:00Z".parse()?,
            end:   "2024-11-20T13:00:00Z".parse()?,
        };
        assert_eq!(format!("{window}"), "Tue 19 Nov 13:00 -- Wed 20 Nov 14:00");
        Ok(())
    }
}
