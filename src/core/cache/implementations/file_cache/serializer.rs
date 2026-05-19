use crate::Result;
use crate::cache::models::CacheEntry;
use std::collections::HashMap;

pub trait CacheEntrySerializer {
    fn serialize(
        &self,
        entries: &HashMap<String, Vec<CacheEntry>>,
    ) -> Result<String>;
    fn deserialize(
        &self,
        text: &str,
    ) -> Result<HashMap<String, Vec<CacheEntry>>>;
}

/// CacheEntry serializer which serializes to a JSON object with URLs as keys
/// and a list of CacheEntries as value:
///
/// {
///  "https://www.dhl.com/tracking/foo": [
///    {
///      "text": "some stringified JSON",
///      "created": "2025-05-17T08:01:05.307751675Z"
///    }
///  ],
///  "https://jouw.postnl.nl/track-and-trace/bar": [
///    {
///      "text": "some stringified JSON",
///      "created": "2026-01-11T21:38:16.037494234Z"
///    },
///    {
///      "text": "some stringified JSON",
///      "created": "2026-01-11T21:39:32.112309418Z"
///    },
///   ]
/// }
pub struct JsonCacheEntrySerializer;
impl CacheEntrySerializer for JsonCacheEntrySerializer {
    fn serialize(
        &self,
        entries: &HashMap<String, Vec<CacheEntry>>,
    ) -> Result<String> {
        Ok(serde_json::to_string_pretty(&entries)?)
    }
    fn deserialize(
        &self,
        text: &str,
    ) -> Result<HashMap<String, Vec<CacheEntry>>> {
        Ok(serde_json::from_str(text)?)
    }
}

#[cfg(test)]
mod tests {

    use crate::utils::UtcTime;

    use super::*;

    fn utc(s: &str) -> UtcTime {
        s.parse().unwrap()
    }

    #[test]
    fn test_json_serialize_deserialize() -> Result<()> {
        // Unfortunately, it is hard to test with more than 1 URL entry, because
        // the order of the HashMap is not guaranteed, and neither is the order
        // of the serialized output.
        let values = HashMap::from([(
            "url1".into(),
            vec![
                CacheEntry {
                    text:    "a".into(),
                    created: utc("2025-05-17T08:01:05.307751675Z"),
                },
                CacheEntry {
                    text:    "b".into(),
                    created: utc("2025-05-18T08:01:05.307751675Z"),
                },
            ],
        )]);
        let expected_serialized = r#"
{
  "url1": [
    {
      "text": "a",
      "created": "2025-05-17T08:01:05.307751675Z"
    },
    {
      "text": "b",
      "created": "2025-05-18T08:01:05.307751675Z"
    }
  ]
}"#
        .trim();

        let serialized = JsonCacheEntrySerializer.serialize(&values)?;
        assert_eq!(serialized, expected_serialized);

        let deserialized =
            JsonCacheEntrySerializer.deserialize(expected_serialized)?;
        assert_eq!(deserialized, values);

        Ok(())
    }
}
