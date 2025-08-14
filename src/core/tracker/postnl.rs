use super::models::{Event, Package, TimeWindow};
use super::tracker::Tracker;
use crate::Result;
use crate::utils::UtcTime;
use async_trait::async_trait;
use regex::Regex;
use serde::Deserialize;
use serde_json::Value;
pub struct PostNLTracker;

#[async_trait]
impl Tracker for PostNLTracker {
    fn can_handle(&self, url: &str) -> bool {
        url.contains("postnl")
    }
    async fn get_raw(
        &self,
        url: &str,
        default_postcode: Option<&str>,
    ) -> Result<String> {
        let barcode = get_barcode(url)?;
        let url = get_url(barcode);
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
fn get_barcode(url: &str) -> Result<String> {
    let rx: Regex = Regex::new(r"https?.*/([A-Z0-9-]+)\??.*").unwrap();
    let barcode = rx
        .captures(url)
        .and_then(|caps| caps.get(1))
        .ok_or(format!("Couldn't get barcode from {url}"))?
        .as_str()
        .to_owned();
    Ok(barcode)
}
fn get_url(barcode: String) -> String {
    format!(
        "https://jouw.postnl.nl/track-and-trace/api/trackAndTrace/{barcode}?language=en"
    )
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
        if let Some(info) = self.route_information.as_ref() {
            return Some(TimeWindow {
                start: info
                    .expected_delivery_time_window
                    .start_date_time,
                end:   info
                    .expected_delivery_time_window
                    .end_date_time,
            });
        }

        if let Some(eta) = self.eta.as_ref() {
            return Some(TimeWindow {
                start: eta.start,
                end:   eta.end,
            });
        }

        return None;
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
    start:  UtcTime,
    end:    UtcTime,
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
    fn test_get_barcode() -> Result<()> {
        let result = get_barcode("");
        assert!(result.is_err());
        assert!(format!("{result:?}").contains("Couldn't get barcode from "));

        let result = get_barcode(
            "https://jouw.postnl.nl/track-and-trace/1ABCDE1234567-AA-1234AB?language=nl",
        )?;
        assert_eq!(result, "1ABCDE1234567-AA-1234AB");
        Ok(())
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
    fn test_get_url() {
        get_url("xxx".into());
    }

    #[test]
    fn test_can_handle() {
        let tracker = PostNLTracker;
        assert_eq!(tracker.can_handle("xxx"), false);
        assert_eq!(tracker.can_handle("jouw.postnl.com/..."), true);
    }
}
