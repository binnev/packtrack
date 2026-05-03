use crate::cli::settings::Settings;
use crate::cli::urls;
use clap::Args;
use clap::Subcommand;
use packtrack::Result;
use packtrack::url_store::AnnotatedUrl;
use packtrack::utils::check_path_exists;
use std::path::PathBuf;

pub async fn handle_url_command(
    command: UrlCommand,
    settings: &Settings,
) -> Result<()> {
    let default_file = &settings.urls_file;
    match command {
        UrlCommand::Add {
            url,
            description,
            args,
        } => {
            let file = args
                .urls_file
                .as_ref()
                .unwrap_or(default_file);
            let msg = format!("Added {url}");
            let aurl = AnnotatedUrl::new(url, description);
            match urls::add(file, aurl) {
                Ok(()) => println!("{msg}"),
                Err(err) => return Err(err),
            }
        }
        UrlCommand::Remove { url, args } => {
            let file = args
                .urls_file
                .as_ref()
                .unwrap_or(default_file);
            match urls::remove(file, url) {
                Ok(removed) => {
                    println!("Removed urls:");
                    for url in removed {
                        println!("{url}");
                    }
                }
                Err(err) => return Err(err),
            }
        }
        UrlCommand::List { query, args } => {
            let file = args
                .urls_file
                .as_ref()
                .unwrap_or(default_file);
            let urls = urls::filter(file, query.as_deref())?;
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
