use crate::cli::display::display_settings;
use clap::Subcommand;
use packtrack::Result;
use packtrack::settings::{FileSettingsManager, Settings, SettingsManager};

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
    settings_manager: &mut FileSettingsManager,
) -> Result<()> {
    match command {
        ConfigCommand::List => display_settings(&settings_manager.settings)?,
        ConfigCommand::Set { key, value } => {
            settings_manager
                .settings
                .update(&key, value)?;
            settings_manager.save()?;
            display_settings(&settings_manager.settings)?;
        }
        ConfigCommand::Reset => {
            settings_manager.settings = Settings::default()?;
            settings_manager.save()?;
            display_settings(&settings_manager.settings)?;
        }
    }
    Ok(())
}
