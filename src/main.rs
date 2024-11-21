#![allow(dead_code)]
#![allow(unused)]

mod api;
mod cli;
mod error;
mod mocks;
mod settings;
mod tracker;
mod urls;

use crate::error::{Error, Result};

#[tokio::main]
async fn main() -> Result<()> {
    cli::main().await
}
