use derive_more::Display;
use packtrack::error::{Error, Result};
use std::{fs, path::PathBuf};

/// Load URLs from file
pub fn load(file: &PathBuf) -> Result<Vec<String>> {
    let urls = fs::read_to_string(file)?
        .lines()
        .map(|s| s.to_owned())
        .collect();
    Ok(urls)
}

/// Save URLs to file
pub fn save(file: &PathBuf, urls: Vec<String>) -> Result<()> {
    fs::write(file, urls.join("\n"))?;
    Ok(())
}

/// Add a URL to the URLs file
pub fn add(file: &PathBuf, url: &str) -> Result<()> {
    log::info!("adding {url}");
    let mut urls = load(file)?;
    add_to_list(&mut urls, url)?;
    save(file, urls)?;
    Ok(())
}

/// Add a URL to a list of URLs, but only if it's not already present in the
/// list
fn add_to_list(urls: &mut Vec<String>, url: &str) -> Result<()> {
    let url = url.into();
    if urls.iter().any(|u| u.contains(&url)) {
        Err(UrlError::AlreadyInFile(url).into())
    } else {
        urls.push(url);
        Ok(())
    }
}
/// Remove a URL from file
pub fn remove(file: &PathBuf, pattern: String) -> Result<Vec<String>> {
    log::info!("removing URLs matching pattern {pattern}");
    let mut urls = load(file)?;
    let removed = remove_from_list(&mut urls, &pattern)?;
    log::info!("removed URLs: {removed:?}");
    save(file, urls)?;
    Ok(removed)
}

/// Remove URLs from a list if they match a pattern. Return an error if the
/// pattern is not found in the list. Return the list of removed URLs if
/// successful. This is a separate function so it's easier to test.
fn remove_from_list(
    urls: &mut Vec<String>,
    pattern: &str,
) -> Result<Vec<String>> {
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

/// Filter URLs from file
pub fn filter(file: &PathBuf, query: Option<&str>) -> Result<Vec<String>> {
    let urls = load(file)?;
    Ok(filter_url_list(urls, query))
}

/// Filter an in-memory list of URLs
fn filter_url_list(urls: Vec<String>, query: Option<&str>) -> Vec<String> {
    match query {
        Some(q) => urls
            .into_iter()
            .filter(|url| url.contains(&q))
            .collect(),
        None => urls,
    }
}

#[derive(Debug, Display)]
pub enum UrlError {
    #[display("'{_0}' is already in the URLs file")]
    AlreadyInFile(String),

    #[display("'{_0}' was not found in the URLs file")]
    NotFound(String),

    #[display("Found multiple URLs that match '{_0}'")]
    MultipleMatches(String),
}
impl From<UrlError> for Error {
    fn from(e: UrlError) -> Error {
        Error::Custom(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn urls() -> Vec<String> {
        vec![
            "www.ups.org".into(),
            "www.example.com".into(),
            "www.dhl.org".into(),
        ]
    }

    #[test]
    fn test_remove_pattern() -> Result<()> {
        let mut urls = urls();
        let removed = remove_from_list(&mut urls, ".org")?;
        assert_eq!(removed, vec!["www.ups.org", "www.dhl.org",]);
        let expected = vec!["www.example.com"];
        assert_eq!(urls, expected);
        Ok(())
    }
    #[test]
    fn test_remove_exact() -> Result<()> {
        let mut urls = urls();
        let removed = remove_from_list(&mut urls, "www.dhl.org")?;
        assert_eq!(removed, vec!["www.dhl.org",]);
        let expected = vec!["www.ups.org", "www.example.com"];
        assert_eq!(urls, expected);
        Ok(())
    }
    #[test]
    fn test_remove_not_found() {
        let mut urls = vec!["www.dhl.org".into()];
        let removed = remove_from_list(&mut urls, "dhl.com");
        assert_eq!(
            removed.err().unwrap(),
            UrlError::NotFound("dhl.com".into()).into()
        );
    }
    #[test]
    fn test_add_happy() -> Result<()> {
        let mut urls = urls();
        add_to_list(&mut urls, "foo.bar")?;
        assert!(urls.contains(&"foo.bar".to_owned()));
        assert_eq!(
            urls,
            vec!["www.ups.org", "www.example.com", "www.dhl.org", "foo.bar"]
        );
        Ok(())
    }
    #[test]
    fn test_add_sad() {
        let mut urls = urls();
        let result = add_to_list(&mut urls, "www.ups.org");
        assert_eq!(
            result.err().unwrap(),
            UrlError::AlreadyInFile("www.ups.org".into()).into()
        );
    }
}
