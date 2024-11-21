use async_trait::async_trait;
use regex::Regex;
use serde::Deserialize;
use serde_json::Value;

use super::{models::UtcTime, tracker::Tracker, Event, Package, TimeWindow};
use crate::Result;

pub struct DhlTracker;

#[async_trait]
impl Tracker for DhlTracker {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("dhl")
    }
    async fn track(&self, url: &str) -> Result<Package> {
        let barcode = get_barcode(url)?;
        let package = track_package(barcode).await?;
        Ok(Package {
            url:        url.to_owned(),
            barcode:    package.barcode.clone(),
            channel:    "DHL".into(),
            sender:     package.sender(),
            recipient:  package.recipient(),
            eta:        package.eta(),
            eta_window: package.eta_window()?,
            delivered:  package.delivered_at,
            events:     package.events(),
        })
    }
}

async fn track_package(barcode: String) -> Result<DhlPackage> {
    let url = get_url(barcode);
    let response = reqwest::get(url).await?;
    let body = response.text().await?;
    let value: Value = serde_json::from_str(&body)?;
    let data = get_first_package(value)?;
    let package: DhlPackage = serde_json::from_value(data.clone())?;
    Ok(package)
}

fn get_barcode(url: &str) -> Result<String> {
    get_dhl_barcode(url).or_else(|_| get_ecommerce_barcode(url))
}

fn get_dhl_barcode(url: &str) -> Result<String> {
    // https://www.dhl.com/nl-en/home/tracking/tracking-parcel.html?locale=true&submit=1&tracking-id=JVGL0614394500301769
    let rx = Regex::new(r".*dhl.com.*tracking-id=([A-Z0-9-].*)")?;
    let barcode = rx
        .captures(url)
        .and_then(|caps| caps.get(1))
        .ok_or(format!("Couldn't get barcode from {url}"))?
        .as_str()
        .to_owned();
    Ok(barcode)
}

fn get_ecommerce_barcode(url: &str) -> Result<String> {
    // https://my.dhlecommerce.nl/home/tracktrace/JJD149990200039892279
    let rx = Regex::new(
        r".*dhlecommerce.*tracktrace/([A-Z0-9-]+)/?([A-Z0-9-]+)?\??.*",
    )?;
    let captures = rx
        .captures(url)
        .ok_or(format!("Couldn't match {url}"))?;

    let barcode = captures
        .get(1)
        .map(|m| m.as_str())
        .ok_or(format!("Couldn't get barcode from {url}"))?
        .to_owned();
    let postcode = captures
        .get(2)
        .map(|m| m.as_str())
        .to_owned();

    let out = if let Some(postcode) = postcode {
        format!("{barcode}%2B{postcode}")
    } else {
        barcode
    };
    Ok(out)
}

fn get_first_package(data: Value) -> Result<Value> {
    let x = data
        .as_array()
        .and_then(|arr| arr.iter().next())
        .ok_or("No packages!")?
        .clone();
    Ok(x)
}
fn get_url(barcode: String) -> String {
    format!("https://api-gw.dhlparcel.nl/track-trace?key={barcode}&role=consumer-receiver")
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DhlPackage {
    barcode:                    String,
    delivered_at:               Option<UtcTime>,
    planned_delivery_timeframe: Option<String>,
    receiver:                   Option<Party>,
    shipper:                    Option<Party>,
    events:                     Vec<DhlEvent>,
    transit_time:               Option<TransitTime>,
}
impl DhlPackage {
    fn events(&self) -> Vec<Event> {
        self.events
            .iter()
            .map(|e| e.to_event())
            .collect()
    }
    fn eta(&self) -> Option<UtcTime> {
        self.transit_time
            .as_ref()
            .map(|t| t.expected_delivery_moment)
    }
    // The Result is because the parsing might fail; the Option is because the
    // data might not be present.
    fn eta_window(&self) -> Result<Option<TimeWindow>> {
        if let Some(s) = &self.planned_delivery_timeframe {
            let window = parse_eta_window(s)?;
            Ok(Some(window))
        } else {
            Ok(None)
        }
    }
    fn sender(&self) -> Option<String> {
        self.shipper
            .as_ref()
            .map(|s| s.name.clone())
    }
    fn recipient(&self) -> Option<String> {
        self.receiver
            .as_ref()
            .map(|r| r.name.clone())
    }
}

fn parse_eta_window(s: &str) -> Result<TimeWindow> {
    let mut parts = s.split("/");
    let (left, right) = parts
        .next()
        .zip(parts.next())
        .ok_or(format!("Couldn't parse EtaWindow {s}"))?;
    Ok(TimeWindow {
        start: left.parse()?,
        end:   right.parse()?,
    })
}
#[derive(Deserialize)]
struct Party {
    name: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TransitTime {
    expected_delivery_moment: UtcTime,
}
#[derive(Deserialize)]
struct DhlEvent {
    timestamp: UtcTime,
    category:  String,
    status:    String,
}
impl DhlEvent {
    fn to_event(&self) -> Event {
        Event {
            timestamp: self.timestamp,
            text:      format!("{}: {}", self.category, self.status),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mocks;

    fn utc(s: &str) -> UtcTime {
        s.parse().unwrap()
    }

    #[test]
    fn test_get_barcode() -> Result<()> {
        for (url, barcode) in [
            (
                "https://my.dhlecommerce.nl/home/tracktrace/3SQLW0022110709/1234AB",
                "3SQLW0022110709%2B1234AB",
            ),
            (
                "https://my.dhlecommerce.nl/home/tracktrace/3SQLW0022110709",
                "3SQLW0022110709",
            ),
            (
                "https://www.dhl.com/nl-en/home/tracking/tracking-parcel.html?locale=true&submit=1&tracking-id=JVGL0614394500301769", 
                "JVGL0614394500301769",
            ),

        ] {
            let result = get_barcode(url)?;
            assert_eq!(result, barcode);
        }

        Ok(())
    }

    #[test]
    fn test_deserialization_undelivered() -> Result<()> {
        let mock = mocks::load_json("dhlecommerce_undelivered_with_postcode")?;
        let data = get_first_package(mock)?;
        let package: DhlPackage = serde_json::from_value(data)?;
        assert_eq!(package.sender().unwrap(), "Sender Name");
        assert_eq!(package.recipient().unwrap(), "Receiver Name");
        assert_eq!(package.barcode, "JVGL06244768002038487552");
        assert_eq!(package.eta().unwrap(), utc("2024-11-07T20:00:00Z"));
        assert_eq!(
            package.eta_window()?.unwrap().start,
            utc("2024-11-08T13:40:00+01:00")
        );
        assert_eq!(
            package.eta_window()?.unwrap().end,
            utc("2024-11-08T15:40:00+01:00")
        );
        assert_eq!(package.delivered_at, None);
        assert_eq!(package.events().len(), 5);
        let event = &package
            .events()
            .into_iter()
            .last()
            .unwrap();
        assert_eq!(event.timestamp, utc("2024-11-08T12:07:05Z"));
        assert_eq!(event.text, "IN_DELIVERY: OUT_FOR_DELIVERY");
        Ok(())
    }
}
