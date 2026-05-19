// Consumer URLs look like:
// https://parcel.trunkrs.nl/419108119/3525EC

// API url:
// https://api.trunkrs.app/v2/tracing/details

// Pass the tracking number and postcode as basic auth header
// 419108119:3525EC -> base64
// Authorization: Basic NDE5MTA4MTE5OjM1MjVFQw==

use crate::Result;
use crate::tracker::{
    Event, Package, PackageStatus, TimeWindow, Tracker, TrackerContext,
};
use crate::utils::UtcTime;
use async_trait::async_trait;
use base64::Engine;
use base64::engine::general_purpose;
use regex::Regex;
use reqwest::header::AUTHORIZATION;
use serde::Deserialize;
use serde_json::Value;
pub struct TrunkrsTracker;

#[async_trait]
impl Tracker for TrunkrsTracker {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("trunkrs")
    }
    async fn get_raw(&self, url: &str, _: &TrackerContext) -> Result<String> {
        let (barcode, postcode) = get_barcode_and_postcode(url)?;
        log::debug!("barcode = {barcode}");
        log::debug!("postcode = {postcode}");
        let pwd = format!("{barcode}:{postcode}");
        log::debug!("pwd = {pwd}");
        let pwd_b64 = general_purpose::STANDARD.encode(pwd);
        log::debug!("pwd_b64 = {pwd_b64}");
        let auth_header = format!("Basic {pwd_b64}");
        let client = reqwest::Client::new();
        let response = client
            .get("https://api.trunkrs.app/v2/tracing/details")
            .header(AUTHORIZATION, auth_header)
            .send()
            .await?
            .error_for_status()?;
        let text = response.text().await?;
        Ok(text)
    }

    fn parse(&self, text: String) -> Result<Package> {
        let value: Value = serde_json::from_str(&text)?;
        let package: TrunkrsPackage = serde_json::from_value(value)?;
        Ok(Package {
            barcode:    package.trunkrs_nr.clone(),
            channel:    "Trunkrs".into(),
            status:     package.status(),
            sender:     package.sender_name.clone(),
            recipient:  package.recipient_name.clone(),
            eta:        None, // TODO
            eta_window: package.time_window(),
            delivered:  package.delivered(),
            events:     package.events(),
        })
    }
}

// https://parcel.trunkrs.nl/419108119/3525EC
fn get_barcode_and_postcode(url: &str) -> Result<(&str, &str)> {
    let rx = Regex::new(
        r".*trunkrs.nl/(?P<barcode>[0-9]+)/(?P<postcode>[0-9A-Za-z]+)",
    )
    .unwrap();
    let captures = rx
        .captures(url)
        .ok_or(format!("Unexpected URL format: {url}"))?;
    let barcode = captures
        .name("barcode")
        .map(|m| m.as_str())
        .ok_or(format!("Couldn't parse barcode from URL: {url}"))?;
    let postcode = captures
        .name("postcode")
        .map(|m| m.as_str())
        .ok_or(format!("Couldn't parse postcode from URL: {url}"))?;
    Ok((barcode, postcode))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TrunkrsPackage {
    trunkrs_nr:        String,
    sender_name:       Option<String>,
    recipient_name:    Option<String>,
    time_slot:         Option<TimeSlot>,
    audit_logs:        Option<Vec<TrunkrsEvent>>,
    delivery_attempts: Option<Vec<DeliveryAttempt>>,
}
impl TrunkrsPackage {
    fn time_window(&self) -> Option<TimeWindow> {
        self.time_slot
            .as_ref()
            .map(|ts| ts.to_time_window())
    }
    fn events(&self) -> Vec<Event> {
        let mut out = Vec::new();
        let logs = match self.audit_logs.clone() {
            Some(logs) => logs,
            None => return out,
        };
        for item in logs {
            match item.to_event() {
                Ok(event) => out.push(event),
                Err(err) => log::warn!("Error parsing Trunkrs event: {err}"),
            }
        }
        out
    }
    fn delivered(&self) -> Option<UtcTime> {
        let delivery_attempts = self.delivery_attempts.clone()?;
        let delivered = delivery_attempts
            .into_iter()
            .filter(|attempt| attempt.state_name == "SHIPMENT_DELIVERED")
            .next()?;
        Some(delivered.set_at)
    }
    fn status(&self) -> PackageStatus {
        match self.delivered() {
            Some(_) => PackageStatus::Delivered,
            _ => PackageStatus::InTransit,
        }
    }
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct TimeSlot {
    from: UtcTime,
    to:   UtcTime,
}
impl TimeSlot {
    fn to_time_window(&self) -> TimeWindow {
        TimeWindow {
            start: self.from,
            end:   self.to,
        }
    }
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
struct TrunkrsEvent {
    created_at: Option<UtcTime>,
    source:     Option<String>,
}
impl TrunkrsEvent {
    fn to_event(&self) -> Result<Event> {
        let created = self
            .created_at
            .clone()
            .ok_or(format!("No createdAt on Trunkrs Event: {self:?}"))?;
        let text = self
            .source
            .clone()
            .ok_or("No description on Trunkrs Event: {self:?}")?;
        Ok(Event {
            timestamp: created,
            text,
        })
    }
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
struct DeliveryAttempt {
    state_name: String,
    set_at:     UtcTime,
}

#[cfg(test)]
mod tests {
    use crate::mocks;

    use super::*;

    fn utc(s: &str) -> UtcTime {
        s.parse().unwrap()
    }

    #[test]
    fn test_deserialize_undelivered() -> Result<()> {
        let mock = mocks::load_text("trunkrs_undelivered.json")?;
        let package = TrunkrsTracker.parse(mock)?;
        assert_eq!(package.barcode, "419108119");
        assert_eq!(package.status, PackageStatus::InTransit);
        assert_eq!(package.sender.unwrap(), "Sender name");
        assert_eq!(package.recipient.unwrap(), "Receiver name");
        let eta_window = package.eta_window.unwrap();
        assert_eq!(eta_window.start, utc("2026-05-18T15:00:00.000Z"));
        assert_eq!(eta_window.end, utc("2026-05-18T20:30:00.000Z"));
        assert_eq!(package.delivered, None);
        Ok(())
    }

    #[test]
    fn test_deserialize_delivered() -> Result<()> {
        let mock = mocks::load_text("trunkrs_delivered.json")?;
        let package = TrunkrsTracker.parse(mock)?;
        assert_eq!(package.barcode, "419108119");
        assert_eq!(package.status, PackageStatus::Delivered);
        assert_eq!(package.sender.unwrap(), "Sender name");
        assert_eq!(package.recipient.unwrap(), "Receiver name");
        let eta_window = package.eta_window.unwrap();
        assert_eq!(eta_window.start, utc("2026-05-18T18:49:02.617Z"));
        assert_eq!(eta_window.end, utc("2026-05-18T19:07:19.169Z"));
        assert_eq!(package.delivered.unwrap(), utc("2026-05-18T18:58:48.771Z"));
        Ok(())
    }
}
