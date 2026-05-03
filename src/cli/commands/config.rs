use crate::cli::settings;
use crate::cli::settings::Settings;
use clap::Subcommand;
use packtrack::Result;

#[derive(Subcommand)]
pub enum ConfigCommand {
    /// List the current settings
    List,
    /// Update the settings
    Set { key: String, value: String },
    /// Reset settings to the defaults
    Reset,
}

pub fn handle_config_command(
    command: ConfigCommand,
    sets: Settings,
) -> Result<()> {
    match command {
        ConfigCommand::List => settings::print()?,
        ConfigCommand::Set { key, value } => {
            let sets = sets.update(&key, value)?;
            settings::save(&sets)?;
        }
        ConfigCommand::Reset => settings::reset()?,
    }
    Ok(())
}
