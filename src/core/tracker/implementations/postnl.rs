use crate::Result;
use crate::tracker::PackageStatus;
use crate::tracker::Tracker;
use crate::tracker::TrackerContext;
use crate::tracker::{Event, Package, TimeWindow};
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
    async fn get_raw(&self, url: &str, ctx: &TrackerContext) -> Result<String> {
        let (barcode, country, url_postcode) = get_barcode_and_postcode(url);
        let url = build_url(
            barcode.ok_or(format!("Couldn't get barcode from {url}"))?,
            country,
            url_postcode.or(ctx.recipient_postcode),
            ctx.language,
        );
        let response = reqwest::get(url)
            .await?
            .error_for_status()?;
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
            status:     package.status(),
            sender:     package.sender(),
            recipient:  package.recipient(),
            eta:        package.eta(),
            eta_window: package.eta_window(),
            delivered:  package.delivery_datetime(),
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
        r"track-and-trace/(?P<barcode>[0-9A-Z]+)(?:[-/](?P<country>[A-Z]{2})[-/](?P<postcode>[0-9A-Z]+))?"
    ).unwrap();

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

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct PostNLPackage {
    barcode:               String,
    sender:                Option<Party>,
    recipient:             Option<Party>,
    delivery_date:         Option<UtcTime>,
    route_information:     Option<RouteInfo>,
    analytics_info:        AnalyticsInfo,
    eta:                   Option<Eta>,
    status_phase:          Option<StatusPhase>,
    delivery_address_type: Option<String>,
    delivery_address:      Option<Party>,
}
impl PostNLPackage {
    fn get_neighbour_address(&self) -> Option<String> {
        if self.delivery_address_type.as_ref()? != "Neighbour" {
            return None;
        }
        let address = self.clone().delivery_address?.address?;
        let street = address.street?;
        let mut number = address.house_number?;
        if let Some(suffix) = address.house_number_suffix {
            number += &suffix;
        }
        Some(format!("{street} {number}"))
    }
    fn status(&self) -> PackageStatus {
        if let Some(_) = self.delivery_datetime() {
            if let Some(address) = self.get_neighbour_address() {
                return PackageStatus::DeliveredToNeighbour { address };
            }
            return PackageStatus::Delivered;
        };

        PackageStatus::InTransit
    }
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
        self.route_information?
            .expected_delivery_time
    }
    fn eta_window(&self) -> Option<TimeWindow> {
        self.eta_window_from_route_info()
            .or(self.eta_window_from_eta())
    }
    fn eta_window_from_route_info(&self) -> Option<TimeWindow> {
        let PostNLTimeWindow {
            start_date_time: s,
            end_date_time: e,
        } = self
            .route_information?
            .expected_delivery_time_window?;
        Some(TimeWindow {
            start: s?,
            end:   e?,
        })
    }
    fn eta_window_from_eta(&self) -> Option<TimeWindow> {
        self.eta
            .as_ref()
            .and_then(|eta| eta.start.zip(eta.end))
            .map(|(start, end)| TimeWindow { start, end })
    }
    fn delivery_datetime(&self) -> Option<UtcTime> {
        // If it exists, return the delivery date
        if self.delivery_date.is_some() {
            return self.delivery_date;
        };

        // If the package is delivered in the letterbox, Postnl doesn't set a
        // delivery date. So instead we try to use the timestamp from the last
        // event.
        let msg = self
            .status_phase
            .as_ref()
            .and_then(|s| s.message.as_ref())?;
        if msg.contains("in letterbox") {
            let dt = get_last_event_datetime(&self.events());
            if dt.is_none() {
                log::warn!(
                    "PostNL {} was delivered in the letterbox, but there are no events we can use to determine the delivery time!",
                    self.barcode
                )
            }
            dt
        } else {
            None
        }
    }
}
fn get_last_event_datetime(events: &Vec<Event>) -> Option<UtcTime> {
    events
        .iter()
        .max_by(|a, b| a.timestamp.cmp(&b.timestamp))
        .map(|e| e.timestamp)
}

#[derive(Deserialize, Clone)]
struct StatusPhase {
    message: Option<String>,
}
#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct AnalyticsInfo {
    all_observations: Vec<PostNLEvent>,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Names {
    company_name: Option<String>,
    person_name:  Option<String>,
}
#[allow(unused)]
#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Address {
    street:              Option<String>,
    house_number:        Option<String>,
    house_number_suffix: Option<String>,
    postal_code:         Option<String>,
    town:                Option<String>,
    country:             Option<String>,
}
impl Address {
    fn format(&self) -> String {
        let mut parts: Vec<&str> = Vec::new();
        if let Some(street) = &self.street {
            parts.push(street)
        }
        if let Some(house_number) = &self.house_number {
            parts.push(house_number);
        }
        parts.join(" ")
    }
}

#[derive(Deserialize, Clone)]
struct Party {
    names:   Names,
    address: Option<Address>,
}
impl Party {
    fn name(&self) -> Option<String> {
        self.names
            .company_name
            .clone()
            .or(self.names.person_name.clone())
            .or(self
                .address
                .as_ref()
                .map(|a| a.format()))
    }
}

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
struct PostNLTimeWindow {
    start_date_time: Option<UtcTime>,
    end_date_time:   Option<UtcTime>,
}

#[derive(Deserialize, Clone)]
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

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
struct RouteInfo {
    expected_delivery_time:        Option<UtcTime>,
    expected_delivery_time_window: Option<PostNLTimeWindow>,
}

#[derive(Deserialize, Clone)]
struct Eta {
    start: Option<UtcTime>,
    end:   Option<UtcTime>,
}

#[cfg(test)]
mod tests {
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
            (
                "https://tracking.postnl.nl/track-and-trace/RN488875617NL-GB-NN188UA",
                Some("RN488875617NL"),
                Some("GB"),
                Some("NN188UA"),
            ),
        ] {
            let (barcode, country, postcode) = get_barcode_and_postcode(url);
            assert_eq!(barcode, expected_barcode);
            assert_eq!(country, expected_country);
            assert_eq!(postcode, expected_postcode);
        }
    }

    #[test]
    fn test_delivered_in_letterbox_gets_delivery_time() -> Result<()> {
        let mock = mocks::load_text("postnl_delivered_in_letterbox.json")?;
        let package = PostNLTracker.parse(mock)?;
        let delivered = package.delivered.unwrap();
        let expected = utc("2025-07-23T19:40:55+02:00");
        assert_eq!(delivered, expected);
        Ok(())
    }
    #[test]
    fn test_alternate_eta_window() -> Result<()> {
        let mock =
            mocks::load_text("postnl_undelivered_but_eta_not_shown.json")?;
        let package = PostNLTracker.parse(mock)?;
        package.eta_window.unwrap();
        Ok(())
    }
    #[test]
    fn test_eta_with_nulls() -> Result<()> {
        let mock = mocks::load_text("postnl_undelivered_eta_with_null.json")?;
        let package = PostNLTracker.parse(mock)?;
        assert!(package.eta_window.is_none());
        Ok(())
    }
    #[test]
    fn test_different_eta() -> Result<()> {
        let mock = mocks::load_text("postnl_undelivered_different_eta.json")?;
        let package = PostNLTracker.parse(mock)?;
        let eta_window = package.eta_window.unwrap();
        assert_eq!(eta_window.start, utc("2025-07-23T08:00:00+02:00"));
        assert_eq!(eta_window.end, utc("2025-07-23T18:00:00+02:00"));
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
    fn test_delivered_to_neighbour() -> Result<()> {
        let mock = mocks::load_text("postnl_delivered_neighbour.json")?;
        let package = PostNLTracker.parse(mock)?;
        assert_eq!(
            package.status,
            PackageStatus::DeliveredToNeighbour {
                address: "Streetname 69b".into(),
            }
        );
        Ok(())
    }

    #[test]
    fn test_deserialization_missing_datetime() -> Result<()> {
        let mock = mocks::load_json("postnl_missing_datetime")?;
        let data = get_first_package(mock)?;
        let _: PostNLPackage = serde_json::from_value(data)?;
        Ok(())
    }

    #[test]
    fn test_deserialization_null_names() -> Result<()> {
        let mock = mocks::load_json("postnl_recipient_null_names")?;
        let data = get_first_package(mock)?;
        let package: PostNLPackage = serde_json::from_value(data)?;
        let recipient = package.recipient().ok_or("")?;
        assert_eq!(recipient, "Streetname 420");
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
