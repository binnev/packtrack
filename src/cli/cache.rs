use crate::cli::display::human_readable_bytes;
use crate::cli::settings::Settings;
use crate::cli::url::UrlArgs;
use crate::cli::urls;
use clap::Subcommand;
use packtrack::Result;
use packtrack::cache::{Cache, JsonCache};
use std::fs;

pub async fn handle_cache_command(
    command: CacheCommand,
    settings: &Settings,
) -> Result<()> {
    match command {
        CacheCommand::Clear => {
            let file = JsonCache::get_file()?;
            if !file.exists() {
                println!("No cache exists currently");
                return Ok(());
            } else {
                let bytes = JsonCache::new()?.size_bytes()?;
                let human_readable = human_readable_bytes(bytes);
                fs::remove_file(file)?;
                println!("Cleared cache (was {human_readable})")
            }
        }
        CacheCommand::Location => {
            println!("{}", JsonCache::get_file()?.display())
        }
        CacheCommand::Size => {
            let cache = JsonCache::new()?;
            let bytes = cache.size_bytes()?;
            println!("{}", human_readable_bytes(bytes))
        }
        CacheCommand::Prune { dry_run, args } => {
            let file = JsonCache::get_file()?;
            if !file.exists() {
                println!("Cache is empty");
                return Ok(());
            }

            let urls_file = args
                .urls_file
                .as_ref()
                .unwrap_or(&settings.urls_file);
            log::info!("Using URLs file {urls_file:#?}");

            let mut cache = JsonCache::new()?;
            let cache_size_before = cache.size_bytes()?;

            let keep: Vec<String> = urls::filter(&urls_file, None)?
                .into_iter()
                .map(|au| au.url)
                .collect();
            log::info!("Aiming to keep {} urls", keep.len());
            for url in keep.iter() {
                log::debug!("Keep {url}");
            }

            let removed_urls = cache.prune(&keep);

            if dry_run {
                println!("Would remove {} urls (dry run)", removed_urls.len());
                for url in removed_urls {
                    log::debug!("Removed {url}");
                }
            } else {
                cache.save().await?;
                let cache_size_after = cache.size_bytes()?;
                println!("Removed {} urls", removed_urls.len());
                for url in &removed_urls {
                    log::debug!("Removed {url}");
                }
                if removed_urls.len() > 0 {
                    println!(
                        "Cache size reduced from {} to {}",
                        human_readable_bytes(cache_size_before),
                        human_readable_bytes(cache_size_after),
                    );
                } else {
                    println!(
                        "Cache size is still {}",
                        human_readable_bytes(cache_size_before)
                    )
                }
            }
        }
    }
    Ok(())
}

#[derive(Subcommand)]
pub enum CacheCommand {
    /// Get the cache size
    Size,
    /// Remove cache entries for URLs that are no longer in the URL store
    Prune {
        /// Perform a dry run without modifying the cache
        #[arg(long)]
        dry_run: bool,
        #[clap(flatten)]
        args:    UrlArgs,
    },
    /// Show where the cache is stored on disk
    Location,
    /// Empty the cache
    Clear,
}
