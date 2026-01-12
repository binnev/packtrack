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
    fn new(path: PathBuf) -> Result<Self> {
        let urls = fs::read_to_string(&path)?
            .lines()
            .map(deserialize)
            .collect::<Result<Vec<_>>>()?;
        Ok(Self { path, urls })
    }
    fn save(&self) -> Result<()> {
        let urls: Vec<String> = self
            .urls
            .iter()
            .cloned()
            .map(|u| u.url)
            .collect();
        fs::write(&self.path, urls.join("\n"))?;
        Ok(())
    }
}
impl UrlStore for SimpleUrlStore {
    fn add(&mut self, entry: AnnotatedUrl) -> Result<()> {
        if let Some(_) = entry.description {
            return Err("SimpleUrlStore doesn't store descriptions".into());
        }
        add_to_list(&mut self.urls, entry.clone());
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
    let mut s = format!("{} | {}", entry.url, entry.created);
    if let Some(d) = &entry.description {
        s += &format!(" | {d}");
    }
    s
}
fn deserialize(s: &str) -> Result<AnnotatedUrl> {
    let parts: Vec<String> = s
        .split("|")
        .take(3)
        .map(|s| s.to_owned())
        .collect();
    let url = parts
        .get(0)
        .map(|s| s.trim().to_owned())
        .ok_or(format!("Couldn't get URL from {s}"))?;
    let created_string = parts
        .get(1)
        .map(|s| s.trim().to_owned())
        .ok_or(format!("Couldn't get created time from {s}"))?;
    let created: DateTime<Utc> = created_string.parse()?;
    let description = parts
        .get(2)
        .map(|d| d.trim().to_owned());
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
            format!("https://example.com | {} | description", url.created)
        );
        let deserialized = deserialize(&s).unwrap();
        assert_eq!(deserialized, url);
    }
    #[test]
    fn test_serialize_deserialize_no_description() {
        let url = AnnotatedUrl::new("https://example.com".to_owned(), None);
        let s = serialize(&url);
        assert_eq!(s, format!("https://example.com | {}", url.created));
        let deserialized = deserialize(&s).unwrap();
        assert_eq!(deserialized, url);
    }
}
