use crate::Result;

pub trait SettingsManager {
    fn save(&self) -> Result<()>;
}
