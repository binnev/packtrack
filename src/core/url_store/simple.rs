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
        let urls = fs::read_to_string(&path)?
            .lines()
            .map(deserialize)
            .collect::<Result<Vec<_>>>()?;
        Ok(Self { path, urls })
    }
    pub fn save(&self) -> Result<()> {
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
    fn test_serialize_deserialize() {
        let url = AnnotatedUrl::new(
            "https://example.com".to_owned(),
            Some("description".to_owned()),
        );
        let s = serialize(&url);
        assert_eq!(
            s,
            format!(
                "https://example.com | {} | description",
                url.created.unwrap()
            )
        );
        let deserialized = deserialize(&s).unwrap();
        assert_eq!(deserialized, url);
    }
    #[test]
    fn test_serialize_deserialize_no_description() {
        let url = AnnotatedUrl::new("https://example.com".to_owned(), None);
        let s = serialize(&url);
        assert_eq!(
            s,
            format!("https://example.com | {}", url.created.unwrap())
        );
        let deserialized = deserialize(&s).unwrap();
        assert_eq!(deserialized, url);
    }
    #[test]
    fn test_serialize_deserialize_no_created() {
        let url = AnnotatedUrl {
            url:         "https://example.com".to_owned(),
            created:     None,
            description: Some("description".to_owned()),
        };
        let s = serialize(&url);
        assert_eq!(s, format!("https://example.com | description"));
        let deserialized = deserialize(&s).unwrap();
        assert_eq!(deserialized, url);
    }
    #[test]
    fn test_serialize_deserialize_only_url() {
        let url = AnnotatedUrl {
            url:         "https://example.com".to_owned(),
            created:     None,
            description: None,
        };
        let s = serialize(&url);
        assert_eq!(s, format!("https://example.com"));
        let deserialized = deserialize(&s).unwrap();
        assert_eq!(deserialized, url);
    }
}
