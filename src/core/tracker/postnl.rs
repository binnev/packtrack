use std::default;

use super::models::{Event, Package, TimeWindow};
use super::tracker::Tracker;
use crate::Result;
use crate::tracker::TrackerContext;
use crate::utils::UtcTime;
use async_trait::async_trait;
use futures::future::AndThen;
use regex::Regex;
use serde::Deserialize;
use serde_json::Value;
pub struct PostNLTracker;

#[async_trait]
impl Tracker for PostNLTracker {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("postnl")
    }
    async fn get_raw(&self, url: &str, ctx: &TrackerContext) -> Result<String> {
        let (barcode, country, url_postcode) = get_barcode_and_postcode(url);
        let url = build_url(
            barcode.ok_or(format!("Couldn't get barcode from {url}"))?,
            country,
            url_postcode.or(ctx.recipient_postcode),
            ctx.language,
        );
        let response = reqwest::get(url).await?;
        let text = response.text().await?;
        Ok(text)
    }

    fn parse(&self, text: String) -> Result<Package> {
        let value: Value = serde_json::from_str(&text)?;
        let data = get_first_package(value)?;
        let package: PostNLPackage = serde_json::from_value(data.clone())?;
        Ok(Package {
            barcode:    package.barcode.clone(),
            channel:    "PostNL".into(),
            sender:     package.sender(),
            recipient:  package.recipient(),
            eta:        package.eta(),
            eta_window: package.eta_window(),
            delivered:  package.delivery_date,
            events:     package.events(),
        })
    }
}

fn get_first_package(data: Value) -> Result<Value> {
    let (_, value) = data
        .get("colli")
        .and_then(|colli| colli.as_object())
        .and_then(|obj| obj.iter().next())
        .ok_or("No packages in payload!")?;
    Ok(value.clone())
}
fn get_barcode_and_postcode(
    url: &str,
) -> (Option<&str>, Option<&str>, Option<&str>) {
    let rx =   Regex::new(
        r"track-and-trace/(?P<barcode>[0-9A-Z]+)(?:[-/](?P<country>[A-Z]{2})[-/](?P<postcode>\d{4}[A-Z]{2}))?"    ).unwrap();

    let mut barcode = None;
    let mut country = None;
    let mut postcode = None;
    if let Some(caps) = rx.captures(url) {
        barcode = caps.name("barcode").map(|m| m.as_str());
        country = caps.name("country").map(|m| m.as_str());
        postcode = caps
            .name("postcode")
            .map(|m| m.as_str());
    }
    log::debug!(
        "PostNL extracted barcode {barcode:?}, country {country:?}, postcode {postcode:?} from url {url}"
    );
    (barcode, country, postcode)
}
fn build_url(
    barcode: &str,
    country: Option<&str>,
    postcode: Option<&str>,
    language: &str,
) -> String {
    let mut barcode = barcode.to_string();

    // Only append the country and postcode if both are present
    if let Some((c, p)) = country.zip(postcode) {
        barcode.push_str(&format!("-{c}-{p}"));
    }
    let url = format!(
        "https://jouw.postnl.nl/track-and-trace/api/trackAndTrace/{barcode}?language={language}"
    );
    log::debug!(
        "Built URL {url} using barcode {barcode:?}, country {country:?}, postcode {postcode:?}"
    );
    url
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PostNLPackage {
    barcode:           String,
    sender:            Option<Party>,
    recipient:         Option<Party>,
    delivery_date:     Option<UtcTime>,
    route_information: Option<RouteInfo>,
    analytics_info:    AnalyticsInfo,
    eta:               Option<Eta>,
}
impl PostNLPackage {
    fn sender(&self) -> Option<String> {
        self.sender
            .as_ref()
            .and_then(|party| party.name())
    }
    fn recipient(&self) -> Option<String> {
        self.recipient
            .as_ref()
            .and_then(|rec| rec.name())
    }
    fn events(&self) -> Vec<Event> {
        self.analytics_info
            .all_observations
            .iter()
            .map(|event| event.to_event())
            .collect()
    }
    fn eta(&self) -> Option<UtcTime> {
        self.route_information
            .as_ref()
            .and_then(|info| Some(info.expected_delivery_time.clone()))
    }
    fn eta_window(&self) -> Option<TimeWindow> {
        self.eta_window_from_route_info()
            .or(self.eta_window_from_eta())
    }
    fn eta_window_from_route_info(&self) -> Option<TimeWindow> {
        self.route_information
            .as_ref()
            .map(|info| TimeWindow {
                start: info
                    .expected_delivery_time_window
                    .start_date_time,
                end:   info
                    .expected_delivery_time_window
                    .end_date_time,
            })
    }
    fn eta_window_from_eta(&self) -> Option<TimeWindow> {
        self.eta
            .as_ref()
            .and_then(|eta| eta.start.zip(eta.end))
            .map(|(start, end)| TimeWindow { start, end })
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AnalyticsInfo {
    all_observations: Vec<PostNLEvent>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Names {
    company_name: Option<String>,
    person_name:  Option<String>,
}

#[derive(Deserialize)]
struct Party {
    names: Names,
}
impl Party {
    fn name(&self) -> Option<String> {
        self.names
            .company_name
            .clone()
            .or(self.names.person_name.clone())
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PostNLTimeWindow {
    start_date_time: UtcTime,
    end_date_time:   UtcTime,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PostNLEvent {
    observation_date: UtcTime,
    description:      String,
}
impl PostNLEvent {
    fn to_event(&self) -> Event {
        Event {
            timestamp: self.observation_date,
            text:      self.description.clone(),
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RouteInfo {
    expected_delivery_time:        UtcTime,
    expected_delivery_time_window: PostNLTimeWindow,
}

#[derive(Deserialize)]
struct Eta {
    r#type: String, // r# allows us to use a keyword as a field name
    start:  Option<UtcTime>,
    end:    Option<UtcTime>,
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};

    use super::*;
    use crate::mocks;

    fn utc(s: &str) -> UtcTime {
        s.parse().unwrap()
    }

    #[test]
    fn test_get_barcode_and_postcode() {
        for (url, expected_barcode, expected_country, expected_postcode) in [
            ("", None, None, None),
            (
                "https://jouw.postnl.nl/track-and-trace/1ABCDE1234567-AA-1234AB?language=nl",
                Some("1ABCDE1234567"),
                Some("AA"),
                Some("1234AB"),
            ),
            (
                "https://jouw.postnl.nl/track-and-trace/1ABCDE1234567/NL/1234AB",
                Some("1ABCDE1234567"),
                Some("NL"),
                Some("1234AB"),
            ),
            (
                "https://jouw.postnl.nl/track-and-trace/1ABCDE1234567",
                Some("1ABCDE1234567"),
                None,
                None,
            ),
            (
                "https://jouw.postnl.nl/track-and-trace/1ABCDE1234567/NL",
                Some("1ABCDE1234567"),
                None,
                None,
            ),
        ] {
            let (barcode, country, postcode) = get_barcode_and_postcode(url);
            assert_eq!(barcode, expected_barcode);
            assert_eq!(country, expected_country);
            assert_eq!(postcode, expected_postcode);
        }
    }

    #[test]
    fn test_deserialization_undelivered() -> Result<()> {
        let mock = mocks::load_json("postnl_undelivered")?;
        let data = get_first_package(mock)?;
        let package: PostNLPackage = serde_json::from_value(data)?;
        assert_eq!(package.sender().unwrap(), "Sender Person");
        assert_eq!(package.recipient().unwrap(), "Recipient Name");
        assert_eq!(package.barcode, "3SPYVS100737499");
        assert_eq!(package.eta().unwrap(), utc("2024-11-06T11:25:00+01:00"));
        assert_eq!(
            package.eta_window().unwrap().start,
            utc("2024-11-06T10:45:00+01:00")
        );
        assert_eq!(
            package.eta_window().unwrap().end,
            utc("2024-11-06T12:05:00+01:00")
        );
        assert_eq!(package.delivery_date, None);
        assert_eq!(package.events().len(), 9);
        let event = &package.events()[0];
        assert_eq!(event.timestamp, utc("2024-11-05T07:50:51.802+01:00"));
        assert_eq!(
            event.text,
            "Shipment expected, but not yet arrived or processed at PostNL"
        );
        Ok(())
    }

    #[test]
    fn test_deserialization_undelivered_eta_with_null() -> Result<()> {
        let mock = mocks::load_json("postnl_undelivered_eta_with_null")?;
        let data = get_first_package(mock)?;
        let package: PostNLPackage = serde_json::from_value(data)?;
        assert_eq!(
            package.recipient(),
            Some("Birkenstock c/o arvato SE".to_string())
        );
        Ok(())
    }

    #[test]
    fn test_deserialization_undelivered_whole_day_eta() -> Result<()> {
        let mock = mocks::load_json("postnl_undelivered_whole_day_eta")?;
        let data = get_first_package(mock)?;
        let package: PostNLPackage = serde_json::from_value(data)?;
        assert_eq!(package.eta(), None);
        assert_eq!(
            package.eta_window().unwrap().start,
            utc("2025-08-14T08:30:00+02:00")
        );
        assert_eq!(
            package.eta_window().unwrap().end,
            utc("2025-08-14T21:30:00+02:00")
        );
        Ok(())
    }
    #[test]
    fn test_deserialization_undelivered_2() -> Result<()> {
        let mock = mocks::load_json("postnl_undelivered_2")?;
        let data = get_first_package(mock)?;
        let package: PostNLPackage = serde_json::from_value(data)?;
        assert_eq!(package.eta().unwrap(), utc("2025-08-14T11:37:00+02:00"));
        assert_eq!(
            package.eta_window().unwrap().start,
            utc("2025-08-14T11:19:00+02:00")
        );
        assert_eq!(
            package.eta_window().unwrap().end,
            utc("2025-08-14T11:55:00+02:00")
        );
        Ok(())
    }
    #[test]
    fn test_deserialization_undelivered_3() -> Result<()> {
        let mock = mocks::load_json("postnl_undelivered_3")?;
        let data = get_first_package(mock)?;
        let package: PostNLPackage = serde_json::from_value(data)?;
        assert_eq!(package.recipient().unwrap(), "Recipient Name");
        Ok(())
    }
    #[test]
    fn test_deserialization_delivered() -> Result<()> {
        let mock = mocks::load_json("postnl_delivered")?;
        let data = get_first_package(mock)?;
        let package: PostNLPackage = serde_json::from_value(data)?;
        assert_eq!(package.sender().unwrap(), "Sender Company");
        assert_eq!(package.recipient().unwrap(), "Recipient Name");
        assert_eq!(package.barcode, "3SIJVT005836083");
        assert_eq!(package.eta(), None);
        assert_eq!(
            package.eta_window().unwrap().start,
            utc("2024-10-29T11:40:00+01:00")
        );
        assert_eq!(
            package.eta_window().unwrap().end,
            utc("2024-10-29T13:40:00+01:00")
        );
        assert_eq!(
            package.delivery_date.unwrap(),
            utc("2024-10-29T11:43:02+01:00")
        );
        assert_eq!(package.events().len(), 13);
        let event = &package.events()[0];
        assert_eq!(event.timestamp, utc("2024-10-27T13:48:58.785+01:00"));
        assert_eq!(
            event.text,
            "Shipment expected, but not yet arrived or processed at PostNL"
        );
        Ok(())
    }

    #[test]
    fn test_deserialization_delivered_no_sender() -> Result<()> {
        let mock = mocks::load_json("postnl_delivered_no_sender")?;
        let data = get_first_package(mock)?;
        let package: PostNLPackage = serde_json::from_value(data)?;
        assert!(package.sender().is_none());
        assert_eq!(package.recipient().unwrap(), "RECIPIENT NAME");
        assert_eq!(package.barcode, "3SDOJB990704220");
        assert_eq!(package.eta(), None);
        assert_eq!(
            package.eta_window().unwrap().start,
            utc("2024-11-16T10:25:00+01:00")
        );
        assert_eq!(
            package.eta_window().unwrap().end,
            utc("2024-11-16T12:25:00+01:00")
        );
        assert_eq!(
            package.delivery_date.unwrap(),
            utc("2024-11-16T10:45:27+01:00")
        );
        assert_eq!(package.events().len(), 12);
        let event = package
            .events()
            .into_iter()
            .last()
            .unwrap();
        assert_eq!(event.timestamp, utc("2024-11-16T10:45:27+01:00"));
        assert_eq!(event.text, "Shipment delivered");
        Ok(())
    }

    #[test]
    fn test_build_url() {
        // bare minimum
        assert_eq!(
            build_url("1ABCDE1234567", None, None, "en"),
            "https://jouw.postnl.nl/track-and-trace/api/trackAndTrace/1ABCDE1234567?language=en"
        );

        // both the country and postcode should be present for them to be added.
        assert_eq!(
            build_url("1ABCDE1234567", None, Some("1234AB"), "en"),
            "https://jouw.postnl.nl/track-and-trace/api/trackAndTrace/1ABCDE1234567?language=en"
        );
        assert_eq!(
            build_url("1ABCDE1234567", Some("NL"), None, "en"),
            "https://jouw.postnl.nl/track-and-trace/api/trackAndTrace/1ABCDE1234567?language=en"
        );

        // fully populated
        assert_eq!(
            build_url("1ABCDE1234567", Some("NL"), Some("1234AB"), "nl"),
            "https://jouw.postnl.nl/track-and-trace/api/trackAndTrace/1ABCDE1234567-NL-1234AB?language=nl"
        );
    }

    #[test]
    fn test_can_handle() {
        let tracker = PostNLTracker;
        assert_eq!(tracker.can_handle("xxx"), false);
        assert_eq!(tracker.can_handle("jouw.postnl.com/..."), true);
    }
}
