use crate::Error;
use crate::Result;
use crate::utils::UtcTime;
use async_trait::async_trait;
use chrono::DateTime;
use chrono::Utc;
use derive_more::Display;
use serde::Deserialize;
use serde::Serialize;
use std::{fs, path::PathBuf};

#[derive(Debug, Display)]
pub enum UrlError {
    #[display("'{_0}' is already in the URL store")]
    AlreadyInStore(String),

    #[display("'{_0}' was not found in the URL store")]
    NotFound(String),
}
