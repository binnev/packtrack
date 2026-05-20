use std::path::PathBuf;

use crate::{
    Result,
    file_handler::{FileHandler, TextFileHandler},
    settings::{
        implementations::file_settings::serializer::{
            JsonSettingsSerializer, SettingsSerializer,
        },
        models::Settings,
        traits::SettingsManager,
    },
};

pub struct FileSettingsManager {
    path:         PathBuf,
    pub settings: Settings,
    file_handler: Box<dyn FileHandler>,
    serializer:   Box<dyn SettingsSerializer>,
}

impl FileSettingsManager {
    /// RAII -- instantiating the struct also loads the settings from file
    pub fn new(path: PathBuf) -> Result<Self> {
        #[allow(unused_mut)]
        let mut file_handler: Box<dyn FileHandler> = Box::new(TextFileHandler);

        // Use a mock file handler in tests to prevent tests doing IO
        #[cfg(test)]
        {
            use crate::file_handler::MockFileHandler;
            file_handler = Box::new(MockFileHandler);
        }

        let serializer = Self::select_serializer(&path)?;
        let text = file_handler.load(&path)?;
        let settings = serializer.deserialize(&text)?;
        Ok(Self {
            path,
            settings,
            file_handler,
            serializer,
        })
    }

    fn select_serializer(
        file: &PathBuf,
    ) -> Result<Box<dyn SettingsSerializer>> {
        if !file.exists() {
            return Err(
                format!("File {} does not exist!", file.display()).into()
            );
        }
        let ext = match file.extension() {
            Some(s) => s.to_str().ok_or(format!(
                "Filename {} is invalid unicode?!",
                file.display()
            ))?,
            None => {
                return Err(format!(
                    "File {} has no extension!",
                    file.display()
                )
                .into());
            }
        };
        let serializer: Box<dyn SettingsSerializer> = match ext {
            "json" => Box::new(JsonSettingsSerializer),
            _ => {
                return Err(format!("Unsupported file extension: {ext}").into());
            }
        };
        Ok(serializer)
    }
}

impl SettingsManager for FileSettingsManager {
    fn save(&self) -> Result<()> {
        let path = self.path.display();
        self.serializer
            .serialize(&self.settings)
            .and_then(|text| self.file_handler.save(&self.path, text))
            .inspect(|_| log::info!("Saved settings to {path}"))
            .inspect_err(|err| {
                log::error!("Error saving settings to {path}: {err}")
            })
    }
}
