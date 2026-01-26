use chrono::{DateTime, Utc};

use crate::{
    Result,
    url_store::{
        UrlStore,
        utils::{add_to_list, filter, remove_from_list},
    },
    utils::save_json,
};
use std::{fs, path::PathBuf};

use crate::{url_store::traits::AnnotatedUrl, utils::load_json};

/// Simple text-based url store with 1 url per line
pub struct SimpleUrlStore {
    path: PathBuf,
    urls: Vec<AnnotatedUrl>,
}
impl SimpleUrlStore {
    pub fn new(path: PathBuf) -> Result<Self> {
        // Don't load from file in tests
        #[cfg(test)]
        return Ok(Self { path, urls: vec![] });

        let urls = fs::read_to_string(&path)?
            .lines()
            .map(deserialize)
            .collect::<Result<Vec<_>>>()?;

        Ok(Self { path, urls })
    }
    pub fn save(&self) -> Result<()> {
        #[cfg(test)]
        return Ok(());

        let urls: Vec<String> = self
            .urls
            .iter()
            .cloned()
            .map(|u| serialize(&u))
            .collect();
        fs::write(&self.path, urls.join("\n"))?;
        Ok(())
    }
}
impl UrlStore for SimpleUrlStore {
    fn add(&mut self, entry: AnnotatedUrl) -> Result<()> {
        add_to_list(&mut self.urls, entry.clone())?;
        self.save()
            .inspect(|_| log::info!("Added URL {entry}"))
    }
    fn remove(&mut self, query: &str) -> Result<Vec<AnnotatedUrl>> {
        let removed = remove_from_list(&mut self.urls, query)?;
        self.save().inspect(|_| {
            log::info!("Removed URLs matching pattern {query}: {removed:#?}")
        });
        Ok(removed)
    }
    fn filter(&self, query: Option<&str>) -> Vec<AnnotatedUrl> {
        filter(&self.urls, query)
    }
}

fn serialize(entry: &AnnotatedUrl) -> String {
    let mut s = format!("{}", entry.url);
    if let Some(c) = &entry.created {
        s += &format!(" | {c}")
    }
    if let Some(d) = &entry.description {
        s += &format!(" | {d}");
    }
    s
}
fn deserialize(s: &str) -> Result<AnnotatedUrl> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_url_store() {
        let mut s = SimpleUrlStore {
            path: "./tmp.txt".into(),
            urls: vec![],
        };
        let url = AnnotatedUrl {
            url:         "example.com".into(),
            description: None,
            created:     None,
        };
        let result = s
            .add(url.clone())
            .expect("The first add should work");
        assert_eq!(s.urls.len(), 1);
        let result = s.add(url.clone());
        assert!(result.is_err())
    }

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
            let s = serialize(&url);
            assert_eq!(s, expected_string, "testcase {description}");
            let deserialized = deserialize(&s).unwrap();
            assert_eq!(deserialized, url, "testcase {description}")
        }
    }
}
