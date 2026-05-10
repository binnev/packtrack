use chrono::{DateTime, Utc};

use crate::{Result, url_store::models::AnnotatedUrl};

pub trait UrlSerializer {
    fn serialize(&self, urls: &Vec<AnnotatedUrl>) -> Result<String>;
    fn deserialize(&self, text: &str) -> Result<Vec<AnnotatedUrl>>;
}

/// URL serializer that serializes to a JSON array of objects:
///
/// [
///   {
///     "url": "https://example.com",
///     "description": "description",
///     "created": "2026-01-26T20:29:30.811840299Z"
///   },
///   {
///     "url": "https://example.com",
///     "description": null,
///     "created": "2026-01-26T20:29:30.811840299Z"
///   },
///   {
///     "url": "https://example.com",
///     "description": "description",
///     "created": null
///   },
///   {
///     "url": "https://example.com",
///     "description": null,
///     "created": null
///   }
/// ]
pub struct JsonUrlSerializer;
impl UrlSerializer for JsonUrlSerializer {
    fn serialize(&self, urls: &Vec<AnnotatedUrl>) -> Result<String> {
        Ok(serde_json::to_string_pretty(&urls)?)
    }
    fn deserialize(&self, text: &str) -> Result<Vec<AnnotatedUrl>> {
        Ok(serde_json::from_str(text)?)
    }
}

/// Simple URL serializer which serializes 1 url per line.
/// The `description` and `created` fields are added separated by "|", if they
/// are present:
///
/// https://example.com | 2026-01-26 20:29:30.811840299 UTC | description
/// https://example.com | 2026-01-26 20:29:30.811840299 UTC
/// https://example.com | description
/// https://example.com
pub struct SimpleUrlSerializer;
impl SimpleUrlSerializer {
    fn serialize_one(&self, entry: &AnnotatedUrl) -> String {
        let mut s = format!("{}", entry.url);
        if let Some(c) = &entry.created {
            s += &format!(" | {c}")
        }
        if let Some(d) = &entry.description {
            s += &format!(" | {d}");
        }
        s
    }
    /// This function does some tricky logic because when there are 2 items
    /// separated by "|", the second one could be `description` or it could be
    /// `added`. So it tries to parse the second value as a datetime. If that
    /// succeeds, it treats it as `created`, and if it fails, it treats it as
    /// `description`.
    fn deserialize_one(&self, s: &str) -> Result<AnnotatedUrl> {
        let parts: Vec<String> = s
            .split("|")
            .take(3)
            .map(|s| s.trim().to_owned())
            .collect();

        let mut created: Option<DateTime<Utc>> = None;
        let mut description: Option<String> = None;
        let url = match parts.len() {
            1 => parts[0].clone(),
            2 => {
                let second = parts[1].clone();
                if let Ok(dt) = second.parse() {
                    created = Some(dt)
                } else {
                    description = Some(second)
                }
                parts[0].clone()
            }
            3 => {
                created = Some(parts[1].parse()?);
                description = Some(parts[2].clone());
                parts[0].clone()
            }
            n => panic!("Unexpected length {n}!"),
        };
        Ok(AnnotatedUrl {
            url,
            description,
            created,
        })
    }
}
impl UrlSerializer for SimpleUrlSerializer {
    fn serialize(&self, urls: &Vec<AnnotatedUrl>) -> Result<String> {
        let serialized_urls: Vec<String> = urls
            .iter()
            .cloned()
            .map(|u| self.serialize_one(&u))
            .collect();
        let text = serialized_urls.join("\n");
        Ok(text)
    }
    fn deserialize(&self, text: &str) -> Result<Vec<AnnotatedUrl>> {
        let urls = text
            .lines()
            .map(|line| self.deserialize_one(line))
            .collect::<Result<Vec<_>>>()?;
        Ok(urls)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_deserialize() {
        let dt: DateTime<Utc> = "2026-01-26 20:29:30.811840299 UTC"
            .parse()
            .unwrap();
        let testcases = [
            (
                "fully populated",
                AnnotatedUrl {
                    url:         "https://example.com".to_owned(),
                    description: Some("description".to_owned()),
                    created:     Some(dt),
                },
                "https://example.com | 2026-01-26 20:29:30.811840299 UTC | description",
            ),
            (
                "no description",
                AnnotatedUrl {
                    url:         "https://example.com".to_owned(),
                    description: None,
                    created:     Some(dt),
                },
                "https://example.com | 2026-01-26 20:29:30.811840299 UTC",
            ),
            (
                "no created time",
                AnnotatedUrl {
                    url:         "https://example.com".to_owned(),
                    description: Some("description".to_owned()),
                    created:     None,
                },
                "https://example.com | description",
            ),
            (
                "only url",
                AnnotatedUrl {
                    url:         "https://example.com".to_owned(),
                    description: None,
                    created:     None,
                },
                "https://example.com",
            ),
        ];
        for (description, url, expected_string) in testcases {
            let s = SimpleUrlSerializer.serialize_one(&url);
            assert_eq!(
                s, expected_string,
                "testcase {description} serialization"
            );
            let deserialized = SimpleUrlSerializer
                .deserialize_one(&s)
                .unwrap();
            assert_eq!(
                deserialized, url,
                "testcase {description} deserialization"
            )
        }
    }

    #[test]
    fn test_json_url_serialize_deserialize() -> Result<()> {
        let dt: DateTime<Utc> = "2026-01-26 20:29:30.811840299 UTC"
            .parse()
            .unwrap();

        let urls = vec![
            AnnotatedUrl {
                url:         "https://example.com".to_owned(),
                description: Some("description".to_owned()),
                created:     Some(dt),
            },
            AnnotatedUrl {
                url:         "https://example.com".to_owned(),
                description: None,
                created:     Some(dt),
            },
            AnnotatedUrl {
                url:         "https://example.com".to_owned(),
                description: Some("description".to_owned()),
                created:     None,
            },
            AnnotatedUrl {
                url:         "https://example.com".to_owned(),
                description: None,
                created:     None,
            },
        ];

        let expected_serialized = r#"
[
  {
    "url": "https://example.com",
    "description": "description",
    "created": "2026-01-26T20:29:30.811840299Z"
  },
  {
    "url": "https://example.com",
    "description": null,
    "created": "2026-01-26T20:29:30.811840299Z"
  },
  {
    "url": "https://example.com",
    "description": "description",
    "created": null
  },
  {
    "url": "https://example.com",
    "description": null,
    "created": null
  }
]
        "#
        .trim();
        let serialized = JsonUrlSerializer
            .serialize(&urls)?
            .trim()
            .to_owned();

        assert_eq!(
            serialized, expected_serialized,
            "serialization should work"
        );

        let deserialized =
            JsonUrlSerializer.deserialize(expected_serialized)?;
        assert_eq!(deserialized, urls, "deserialization should work");

        Ok(())
    }
    #[test]
    fn test_simple_url_serialize_deserialize() -> Result<()> {
        let dt: DateTime<Utc> = "2026-01-26 20:29:30.811840299 UTC"
            .parse()
            .unwrap();

        let urls = vec![
            AnnotatedUrl {
                url:         "https://example.com".to_owned(),
                description: Some("description".to_owned()),
                created:     Some(dt),
            },
            AnnotatedUrl {
                url:         "https://example.com".to_owned(),
                description: None,
                created:     Some(dt),
            },
            AnnotatedUrl {
                url:         "https://example.com".to_owned(),
                description: Some("description".to_owned()),
                created:     None,
            },
            AnnotatedUrl {
                url:         "https://example.com".to_owned(),
                description: None,
                created:     None,
            },
        ];
        let expected_serialized = "
https://example.com | 2026-01-26 20:29:30.811840299 UTC | description
https://example.com | 2026-01-26 20:29:30.811840299 UTC
https://example.com | description
https://example.com
        "
        .trim();

        let serialized = SimpleUrlSerializer
            .serialize(&urls)?
            .trim()
            .to_owned();
        assert_eq!(
            expected_serialized, serialized,
            "serialization should work"
        );

        let deserialized =
            SimpleUrlSerializer.deserialize(expected_serialized)?;
        assert_eq!(deserialized, urls, "deserialization should work");

        Ok(())
    }
}
