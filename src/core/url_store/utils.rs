use crate::Result;
use crate::url_store::traits::{AnnotatedUrl, UrlError};

/// Add a URL to a list of URLs, but only if it's not already present in the
/// list
pub fn add_to_list(
    urls: &mut Vec<AnnotatedUrl>,
    url: AnnotatedUrl,
) -> Result<()> {
    if urls
        .iter()
        .any(|u| u.url.contains(&url.url))
    {
        Err(UrlError::AlreadyInStore(url.url).into())
    } else {
        urls.push(url);
        Ok(())
    }
}

/// Remove URLs from a list if they match a pattern. Return an error if the
/// pattern is not found in the list. Return the list of removed URLs if
/// successful. This is a separate function so it's easier to test.
pub fn remove_from_list(
    urls: &mut Vec<AnnotatedUrl>,
    pattern: &str,
) -> Result<Vec<AnnotatedUrl>> {
    let mut removed: Vec<AnnotatedUrl> = vec![];
    while let Some(idx) = urls
        .iter()
        .position(|x| x.url.contains(&pattern))
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

/// Filter an in-memory list of URLs.
pub fn filter<'a>(
    urls: &'a Vec<AnnotatedUrl>,
    query: Option<&str>,
) -> Vec<AnnotatedUrl> {
    match query {
        Some(q) => urls
            .into_iter()
            .filter(|url| url.url.contains(&q))
            .cloned()
            .collect(),
        None => urls.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test utilities
    impl Into<AnnotatedUrl> for &str {
        fn into(self) -> AnnotatedUrl {
            let url = self.to_string();
            AnnotatedUrl::new(url, None)
        }
    }
    impl PartialEq<&str> for AnnotatedUrl {
        fn eq(&self, other: &&str) -> bool {
            self.url == *other
        }
    }

    fn urls() -> Vec<AnnotatedUrl> {
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
        assert_eq!(removed, vec!["www.ups.org", "www.dhl.org"]);
        assert_eq!(urls, vec!["www.example.com"]);
        Ok(())
    }
    #[test]
    fn test_remove_exact() -> Result<()> {
        let mut urls = urls();
        let removed = remove_from_list(&mut urls, "www.dhl.org")?;
        assert_eq!(removed, vec!["www.dhl.org"]);
        assert_eq!(urls, vec!["www.ups.org", "www.example.com"]);
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
        add_to_list(&mut urls, "foo.bar".into())?;
        assert_eq!(
            urls,
            vec!["www.ups.org", "www.example.com", "www.dhl.org", "foo.bar"]
        );
        Ok(())
    }
    #[test]
    fn test_add_sad() {
        let mut urls = urls();
        let result = add_to_list(&mut urls, "www.ups.org".into());
        assert_eq!(
            result.err().unwrap(),
            UrlError::AlreadyInStore("www.ups.org".into()).into()
        );
    }
}
