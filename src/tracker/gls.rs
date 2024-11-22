use async_trait::async_trait;
use chrono::{NaiveDateTime, TimeZone, Utc};
use log;
use regex::Regex;
use serde::Deserialize;
use serde_json::Value;

use super::{models::UtcTime, tracker::Tracker, Event, Package, TimeWindow};
use crate::{Error, Result};
pub struct GlsTracker;

#[async_trait]
impl Tracker for GlsTracker {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("www.gls")
    }
    async fn track(&self, url: &str) -> Result<Package> {
        let (barcode, postcode) = get_barcode_postcode(url)?;
        let url = get_url(&barcode, &postcode);
        let response = reqwest::get(&url).await?;
        let text = response.text().await?;
        let data: Value = serde_json::from_str(&text).map_err(|err| {
            format!("Error parsing request.text to JSON: {err}")
        })?;
        let package = parse_package(data, &url)?;
        Ok(package)
    }
}
#[derive(Deserialize, Default, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
struct GlsPackage {
    parcel_no:       Option<String>,
    address_info:    Option<AddressInfo>,
    delivery_status: Option<DeliveryStatus>,
    scans:           Option<Vec<GlsEvent>>,
}
impl GlsPackage {
    fn events(&self) -> Result<Vec<Event>> {
        let mut events = vec![];
        if let Some(scans) = &self.scans {
            for scan in scans.iter() {
                let event = scan.to_event()?;
                events.push(event);
            }
        }
        Ok(events)
    }
    fn eta(&self) -> Option<UtcTime> {
        self.delivery_status
            .as_ref()
            .and_then(|status| status.eta_timestamp)
            .map(|naive| naive.and_utc())
    }
    fn eta_window(&self) -> Option<TimeWindow> {
        self.delivery_status
            .as_ref()
            .and_then(|status| {
                status
                    .eta_timestamp_min
                    .zip(status.eta_timestamp_max)
            })
            .map(|(start, end)| TimeWindow {
                start: start.and_utc(),
                end:   end.and_utc(),
            })
    }
    fn sender(&self) -> Option<String> {
        self.address_info
            .as_ref()
            .and_then(|x| x.from.as_ref())
            .and_then(|x| x.name.clone())
            .filter(|name| !name.is_empty()) // convert "" to None
    }
    fn recipient(&self) -> Option<String> {
        self.address_info
            .as_ref()
            .and_then(|x| x.recipient.as_ref())
            .and_then(|x| x.name.clone())
            .filter(|name| !name.is_empty()) // convert "" to None
    }
    fn to_package(&self, url: &str) -> Result<Package> {
        Ok(Package {
            url:        url.to_owned(),
            barcode:    self
                .parcel_no
                .clone()
                .ok_or("No barcode!")?,
            channel:    "GLS".into(),
            sender:     self.sender(),
            recipient:  self.recipient(),
            eta:        self.eta(),
            eta_window: self.eta_window(),
            delivered:  None, // TODO: Don't know what this looks like yet
            events:     self.events()?,
        })
    }
}

#[derive(Deserialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
struct GlsEvent {
    date_time:          Option<NaiveDateTime>,
    event_reason_descr: Option<String>,
}
impl GlsEvent {
    fn to_event(&self) -> Result<Event> {
        let timestamp = self
            .date_time
            .ok_or("No datetime on event!")?
            .and_utc();
        let text = self
            .event_reason_descr
            .clone()
            .ok_or("No event description!")?;
        Ok(Event { timestamp, text })
    }
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
struct DeliveryStatus {
    eta_timestamp:     Option<NaiveDateTime>,
    eta_timestamp_max: Option<NaiveDateTime>,
    eta_timestamp_min: Option<NaiveDateTime>,
}
#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
struct AddressInfo {
    from:      Option<Party>,
    recipient: Option<Party>,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
struct Party {
    name: Option<String>,
}
fn parse_package(data: Value, url: &str) -> Result<Package> {
    let package: GlsPackage = serde_json::from_value(data.clone())?;
    log::info!("Successfully parsed package from {url}");
    package.to_package(url)
}
fn get_barcode_postcode(url: &str) -> Result<(String, String)> {
    // https://www.gls-info.nl/tracking?parcelNo=123412341234&zipcode=1234AB
    log::info!("Parsing GLS url {url}");
    let rx = Regex::new(
        r"https://www.gls.*parcelNo=([A-Z0-9]+)&zipcode=([A-Z0-9]+)",
    )?;

    let captures = rx
        .captures(url)
        .ok_or(format!("Couldn't get barcode from {url}"))?;

    let barcode = captures
        .get(1)
        .map(|m| m.as_str())
        .ok_or(format!("Couldn't get barcode from url {url}"))?
        .to_owned();
    log::info!("Parsed barcode {barcode}");
    let postcode = captures
        .get(2)
        .map(|m| m.as_str())
        .ok_or("Couldn't get postcode from {url}")?
        .to_owned();
    log::info!("Parsed postcode {postcode}");
    Ok((barcode, postcode))
}

fn get_url(barcode: &str, postcode: &str) -> String {
    format!("https://apm.gls.nl/api/tracktrace/v1/{barcode}/postalcode/{postcode}/details/en-GB")
}

#[cfg(test)]
mod tests {

    use serde_json::json;

    use super::*;
    use crate::mocks;

    fn utc(s: &str) -> UtcTime {
        s.parse().unwrap()
    }

    #[allow(non_upper_case_globals)]
    const url: &str = "www.example.com";

    #[test]
    fn test_deserialize_empty_gls_package() {
        let data = json!({});
        let gls_package: GlsPackage =
            serde_json::from_value(data.clone()).unwrap();
        assert_eq!(gls_package, GlsPackage::default());
    }

    /// The barcode is the only piece of information we should raise an error
    /// for if it is missing.
    #[test]
    fn test_to_package_sad() {
        let pack = GlsPackage::default();
        assert_eq!(
            pack.to_package(url).err().unwrap(),
            Error::from("No barcode!")
        );
    }

    #[test]
    fn test_deserialization_minimal() {
        let data = json!({"parcelNo": "1234"});
        parse_package(data, url).unwrap();
    }
    #[test]
    fn test_deserialization_error_event() {
        let data = json!({
            "parcelNo": "1234",
            "scans": [{}]
        });
        assert_eq!(
            parse_package(data, url).err().unwrap(),
            Error::from("No datetime on event!")
        );

        let data = json!({
            "parcelNo": "1234",
            "scans": [{"dateTime": "foo"}]
        });
        assert!(parse_package(data, url)
            .err()
            .unwrap()
            .to_string()
            .contains("input contains invalid characters")); // TODO: this is so
                                                             // vague
    }

    #[test]
    fn test_deserialize_undelivered() -> Result<()> {
        let data = mocks::load_json("gls_undelivered")?;
        let package = parse_package(data, url)?;
        assert_eq!(package.sender.unwrap(), "Sender Name");
        assert_eq!(package.recipient, None);
        assert_eq!(package.barcode, "57250013150034");
        assert_eq!(package.eta, None);
        assert_eq!(package.eta_window, None);
        assert_eq!(package.events.len(), 1);
        let event = package
            .events
            .into_iter()
            .last()
            .unwrap();
        assert_eq!(event.timestamp, utc("2024-11-20T10:00:07.226Z"));
        assert_eq!(event.text, "The parcel data was entered into the GLS IT system; the parcel was not yet handed over to GLS.");
        Ok(())
    }
    #[test]
    fn test_deserialize_undelivered_with_eta() -> Result<()> {
        let data = mocks::load_json("gls_undelivered_with_eta")?;
        let package = parse_package(data, url)?;
        assert_eq!(package.sender.unwrap(), "Sender Name");
        assert_eq!(package.recipient, None);
        assert_eq!(package.barcode, "57250013150034");
        assert_eq!(package.eta.unwrap(), utc("2024-11-21T08:15:00Z"));
        assert_eq!(
            package.eta_window.unwrap(),
            TimeWindow {
                start: utc("2024-11-21T08:15:00Z"),
                end:   utc("2024-11-21T10:15:00Z"),
            }
        );
        assert_eq!(package.events.len(), 3);
        let event = package
            .events
            .into_iter()
            .last()
            .unwrap();
        assert_eq!(event.timestamp, utc("2024-11-20T20:17:02.051Z"));
        assert_eq!(event.text, "The parcel has left the parcel center.");
        Ok(())
    }
    #[test]
    fn test_deserialize_undelivered_3() -> Result<()> {
        let data = mocks::load_json("gls_undelivered_3")?;
        let package = parse_package(data, url)?;
        assert_eq!(package.sender.unwrap(), "Sender Name");
        assert_eq!(package.recipient, None);
        assert_eq!(package.barcode, "57250013150034");
        assert_eq!(package.eta.unwrap(), utc("2024-11-21T08:15:00Z"));
        assert_eq!(
            package.eta_window.unwrap(),
            TimeWindow {
                start: utc("2024-11-21T08:15:00Z"),
                end:   utc("2024-11-21T10:15:00Z"),
            }
        );
        assert_eq!(package.events.len(), 5);
        let event = package
            .events
            .into_iter()
            .last()
            .unwrap();
        assert_eq!(event.timestamp, utc("2024-11-21T07:59:04Z"));
        assert_eq!(
            event.text,
            "The parcel is expected to be delivered during the day."
        );
        Ok(())
    }
    #[test]
    fn test_deserialize_delivered() -> Result<()> {
        let data = mocks::load_json("gls_delivered")?;
        let package = parse_package(data, url)?;
        assert_eq!(package.sender.unwrap(), "Sender Name");
        assert_eq!(package.recipient, None);
        assert_eq!(package.barcode, "57250013150034");
        assert_eq!(package.eta, None);
        assert_eq!(package.eta_window, None);
        assert_eq!(package.events.len(), 11);
        let event = package
            .events
            .into_iter()
            .last()
            .unwrap();
        assert_eq!(event.timestamp, utc("2024-11-22T08:28:43Z"));
        assert_eq!(event.text, "The parcel has been delivered.");
        Ok(())
    }
}
