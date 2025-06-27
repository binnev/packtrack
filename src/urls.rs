/// URL management
use crate::error::{Error, Result};
use crate::settings;
use derive_more::Display;
use std::ops::Index;
use std::path::Path;
use std::{env, fs};

#[derive(Debug, Display)]
pub enum UrlError {
    #[display("'{_0}' is already in the URLs file")]
    AlreadyInFile(String),

    #[display("'{_0}' was not found in the URLs file")]
    NotFound(String),

    #[display("Found multiple URLs that match '{_0}'")]
    MultipleMatches(String),
}

pub async fn add(url: &str) -> Result<()> {
    log::info!("adding {url}");
    let mut urls = load()?;
    _add(&mut urls, url)?;
    save(urls)?;
    Ok(())
}
fn _add(urls: &mut Vec<String>, url: &str) -> Result<()> {
    let url = url.into();
    if urls.iter().any(|u| u.contains(&url)) {
        Err(UrlError::AlreadyInFile(url).into())
    } else {
        urls.push(url);
        Ok(())
    }
}
pub async fn remove(pattern: String) -> Result<Vec<String>> {
    log::info!("removing URLs matching pattern {pattern}");
    let mut urls = load()?;
    let removed = _remove(&mut urls, &pattern)?;
    log::info!("removed URLs: {removed:?}");
    save(urls)?;
    Ok(removed)
}
// making this a separate function so it's easier to test
fn _remove(urls: &mut Vec<String>, pattern: &str) -> Result<Vec<String>> {
    let mut removed: Vec<String> = vec![];
    while let Some(idx) = urls
        .iter()
        .position(|x| x.contains(&pattern))
    {
        let url = urls.remove(idx);
        log::debug!("Removed URL: {url}");
        removed.push(url);
    }
    if removed.len() == 0 {
        Err(UrlError::NotFound(pattern.into()).into())
    } else {
        Ok(removed)
    }
}

pub async fn list(query: Option<String>) -> Result<()> {
    let mut urls = load()?;
    if let Some(s) = query {
        urls = urls
            .into_iter()
            .filter(|url| url.contains(&s))
            .collect();
    }

    for url in urls {
        println!("{url}");
    }
    Ok(())
}

pub fn filter(query: Option<&str>) -> Result<Vec<String>> {
    let mut urls = load()?;
    Ok(match query {
        Some(q) => urls
            .into_iter()
            .filter(|url| url.contains(&q))
            .collect(),
        None => urls,
    })
}

pub fn find_one(query: &str) -> Result<Option<String>> {
    let urls = load()?;
    let url = _find_one(urls, query)?;
    Ok(url)
}
fn _find_one(urls: Vec<String>, query: &str) -> Result<Option<String>> {
    let mut matches = urls
        .into_iter()
        .filter(|url| url.contains(&query));
    let url = matches.next();
    if let Some(other_match) = matches.next() {
        Err(UrlError::MultipleMatches(query.to_owned()).into())
    } else {
        Ok(url)
    }
}

/// Load all URLs from the URLs file.
pub fn load() -> Result<Vec<String>> {
    let urls_file = settings::load()?.urls_file;
    let urls = fs::read_to_string(urls_file)?
        .lines()
        .map(|s| s.to_owned())
        .collect();
    Ok(urls)
}

pub fn save(urls: Vec<String>) -> Result<()> {
    let file = settings::load()?.urls_file;
    fs::write(file, urls.join("\n"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::error::Error;

    use super::*;

    fn urls() -> Vec<String> {
        vec![
            "www.example.com".into(),
            "www.ups.org".into(),
            "www.dhl.org".into(),
        ]
    }

    #[test]
    fn test_remove_pattern() -> Result<()> {
        let mut urls = urls();
        let removed = _remove(&mut urls, ".org")?;
        assert_eq!(removed, vec!["www.ups.org", "www.dhl.org",]);
        let expected = vec!["www.example.com"];
        assert_eq!(urls, expected);
        Ok(())
    }
    #[test]
    fn test_remove_exact() -> Result<()> {
        let mut urls = urls();
        let removed = _remove(&mut urls, "www.dhl.org")?;
        assert_eq!(removed, vec!["www.dhl.org",]);
        let expected = vec!["www.example.com", "www.ups.org"];
        assert_eq!(urls, expected);
        Ok(())
    }
    #[test]
    fn test_remove_not_found() {
        let mut urls = vec!["www.dhl.org".into()];
        let removed = _remove(&mut urls, "dhl.com");
        assert_eq!(
            removed.err().unwrap(),
            UrlError::NotFound("dhl.com".into()).into()
        );
    }
    #[test]
    fn test_add_happy() -> Result<()> {
        let mut urls = urls();
        _add(&mut urls, "foo.bar")?;
        assert!(urls.contains(&"foo.bar".to_owned()));
        assert_eq!(
            urls,
            vec!["www.example.com", "www.ups.org", "www.dhl.org", "foo.bar"]
        );
        Ok(())
    }
    #[test]
    fn test_add_sad() {
        let mut urls = urls();
        let result = _add(&mut urls, "www.ups.org");
        assert_eq!(
            result.err().unwrap(),
            UrlError::AlreadyInFile("www.ups.org".into()).into()
        );
    }

    #[test]
    fn test_find_one() {
        assert_eq!(_find_one(vec![], "foo"), Ok(None));
        assert_eq!(_find_one(urls(), "ups"), Ok(Some("www.ups.org".into())));
        assert_eq!(
            _find_one(urls(), "org"),
            Err(UrlError::MultipleMatches("org".into()).into())
        );
    }
}
