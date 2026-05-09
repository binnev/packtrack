use chrono::{DateTime, Utc};

use crate::{
    Result,
    file_handler::{FileHandler, TextFileHandler},
    url_store::{
        UrlStore,
        models::AnnotatedUrl,
        utils::{add_to_list, filter, remove_from_list},
    },
};
use std::path::PathBuf;

/// Simple text-based url store with 1 url per line
pub struct SimpleUrlStore {
    path:         PathBuf,
    file_handler: Box<dyn FileHandler>,
    urls:         Vec<AnnotatedUrl>,
}
impl SimpleUrlStore {
    pub fn new(path: PathBuf) -> Result<Self> {
        #[allow(unused_mut)]
        let mut file_handler: Box<dyn FileHandler> = Box::new(TextFileHandler);

        #[cfg(test)]
        {
            use crate::file_handler::MockFileHandler;
            file_handler = Box::new(MockFileHandler);
        }

        let urls = file_handler
            .load(&path)?
            .lines()
            .map(deserialize)
            .collect::<Result<Vec<_>>>()?;

        Ok(Self {
            path,
            file_handler,
            urls,
        })
    }
}
impl UrlStore for SimpleUrlStore {
    fn add(&mut self, entry: AnnotatedUrl) -> Result<()> {
        add_to_list(&mut self.urls, entry.clone())?;
        log::info!("Added URL {entry}");
        Ok(())
    }
    fn remove(&mut self, query: &str) -> Result<Vec<AnnotatedUrl>> {
        let removed = remove_from_list(&mut self.urls, query)?;
        log::info!(
            "Removed {} URLs matching pattern {query}: {removed:#?}",
            removed.len()
        );
        Ok(removed)
    }
    fn filter(&self, query: Option<&str>) -> Vec<AnnotatedUrl> {
        filter(&self.urls, query)
    }
    fn save(&self) -> Result<()> {
        let urls: Vec<String> = self
            .urls
            .iter()
            .cloned()
            .map(|u| serialize(&u))
            .collect();
        self.file_handler
            .save(&self.path, urls.join("\n"))?;
        Ok(())
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
    fn test_simple_url_store() -> Result<()> {
        let mut s = SimpleUrlStore::new("/dev/null".into())?;
        let url = AnnotatedUrl {
            url:         "example.com".into(),
            description: None,
            created:     None,
        };
        s.add(url.clone())
            .expect("The first add should work");
        assert_eq!(s.urls.len(), 1);
        let result = s.add(url.clone());
        assert!(result.is_err());
        Ok(())
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
