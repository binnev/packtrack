use crate::cli::settings::Settings;
use clap::Args;
use clap::Subcommand;
use packtrack::Result;
use packtrack::url_store::{AnnotatedUrl, FileUrlStore, UrlStore};
use packtrack::utils::check_path_exists;
use std::path::PathBuf;

pub async fn handle_url_command(
    command: UrlCommand,
    settings: &Settings,
) -> Result<()> {
    let default_file = &settings.urls_file;
    let file = match &command {
        UrlCommand::Add { args, .. } => args,
        UrlCommand::Remove { args, .. } => args,
        UrlCommand::List { args, .. } => args,
    }
    .urls_file
    .as_ref()
    .unwrap_or(default_file);
    let mut url_store = FileUrlStore::new(file.clone())?;

    match command {
        UrlCommand::Add {
            url, description, ..
        } => {
            let msg = format!("Added {url}");
            let aurl = AnnotatedUrl::new(url, description);
            url_store.add(aurl)?;
            url_store.save()?;
            println!("{msg}");
        }
        UrlCommand::Remove { url, .. } => {
            let removed = url_store.remove(&url)?;
            url_store.save()?;
            println!("Removed urls:");
            for url in removed {
                println!("{url}");
            }
        }
        UrlCommand::List { query, .. } => {
            let urls = url_store.filter(query.as_deref());
            for url in urls {
                println!("{url}");
            }
        }
    }
    Ok(())
}

#[derive(Subcommand)]
pub enum UrlCommand {
    /// List the URLs currently in the file
    List {
        query: Option<String>,
        #[clap(flatten)]
        args:  UrlArgs,
    },
    /// Add a URL to the urls file
    Add {
        url:         String,
        #[arg(short, long)]
        description: Option<String>,
        #[clap(flatten)]
        args:        UrlArgs,
    },
    /// Remove a URL from the urls file
    Remove {
        url:  String,
        #[clap(flatten)]
        args: UrlArgs,
    },
}

#[derive(Args)]
pub struct UrlArgs {
    /// Path to the URLs file
    #[arg(short, long, value_parser = check_path_exists)]
    pub urls_file: Option<PathBuf>,
}
