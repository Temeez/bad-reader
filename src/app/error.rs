use thiserror::Error;


#[derive(Error, Debug)]
pub enum SettingsError {
    #[error("Failed to open and read config file on disk.")]
    ReadFromDisk,
    #[error("Failed to write config file to disk.")]
    WriteToDisk,
    #[error("Unknown error.")]
    Unknown,
}

impl From<anyhow::Error> for SettingsError {
    fn from(_err: anyhow::Error) -> Self {
        SettingsError::Unknown
    }
}