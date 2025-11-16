use chrono_tz::{Europe::Amsterdam, Tz};
use std::fmt::Display;

use chrono::{DateTime, Datelike, Local, TimeZone, Utc};
use enum_iterator::Sequence;

use crate::utils::UtcTime;

#[derive(Debug, Clone)]
pub struct Package {
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeWindow {
    pub start: UtcTime,
    pub end:   UtcTime,
}
#[derive(Debug, Clone)]
pub struct Event {
    pub timestamp: UtcTime,
    pub text:      String,
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

/// Contains the configurable stuff for Tracker
#[derive(Clone)]
pub struct TrackerContext<'a> {
    /// Postcode of the recipient (sometimes necessary to get full data from
    /// the API)
    pub recipient_postcode: Option<&'a str>,
    /// Preferred language (usually passed as a query param to the API)
    pub language:           &'a str,
}
