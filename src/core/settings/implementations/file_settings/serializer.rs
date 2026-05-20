use std::collections::HashMap;

use serde_json::Value;

use crate::Result;
use crate::settings::models::Settings;

pub trait SettingsSerializer {
    fn serialize(&self, settings: &Settings) -> Result<String>;
    fn deserialize(&self, text: &str) -> Result<Settings>;
}

pub struct JsonSettingsSerializer;
impl SettingsSerializer for JsonSettingsSerializer {
    fn serialize(&self, settings: &Settings) -> Result<String> {
        Ok(serde_json::to_string_pretty(&settings)?)
    }
    fn deserialize(&self, text: &str) -> Result<Settings> {
        // The values stored in the file might be incomplete
        let possibly_incomplete: HashMap<String, Value> =
            serde_json::from_str(text)?;
        // use defaults as a fallback
        let mut defaults = serde_json::to_value(Settings::default()?)?
            .as_object()
            .ok_or("Couldn't cast default settings to HashMap?!")?
            .clone();
        defaults.extend(possibly_incomplete);
        let settings: Settings =
            serde_json::from_value(Value::Object(defaults))?;

        Ok(settings)
    }
}
