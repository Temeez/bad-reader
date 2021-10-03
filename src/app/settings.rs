use crate::app::error::SettingsError;
use std::path::{Path, PathBuf};
use std::io::{BufWriter, BufReader};
use std::fs::File;
use std::fmt;
use anyhow::{Context};
use bincode::{deserialize_from, serialize_into};
use serde::{Deserialize, Serialize};
use crate::ui::{Theme};
use crate::app::utils::working_dir;
use std::str::FromStr;


static CONFIG_NAME: &str = "bad-reader.conf";

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename = "settings")]
pub struct Settings {
    pub general: GeneralSettings,
    pub file: FileSettings,
    path: PathBuf,
}

impl Settings {
    pub fn open() -> Result<Self, SettingsError> {
        let path = Path::new(&working_dir(CONFIG_NAME)).to_path_buf();
        if path.as_path().exists() {
            let f = File::open(&path)
                .context(SettingsError::ReadFromDisk)?;
            if let Ok(settings) = deserialize_from(BufReader::new(&f))
                .context(SettingsError::Unknown) {
                return Ok(settings);
            }
        }
        
        // If file doesn't exist then set then create it
        let settings = Settings {
            general: GeneralSettings::new(),
            file: FileSettings::new(),
            path
        };
        
        settings.write().context(SettingsError::WriteToDisk)?;
        Ok(settings)
    }
    
    pub fn write(&self) -> Result<(), SettingsError> {
        let mut f = BufWriter::new(File::create(&self.path).unwrap());
        serialize_into(&mut f, self)
            .context(SettingsError::WriteToDisk)?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct GeneralSettings {
    pub theme: Theme,
    pub font_size: f64,
    pub font_family: String,
    pub use_custom_color: bool,
    pub background_color: String,
    pub text_color: String,
    pub auto_scroll_speed: f64,
    pub show_page_num: bool,
    pub show_page_file: bool,
}

impl GeneralSettings {
    fn new() -> GeneralSettings {
        GeneralSettings {
            theme: Theme::Sepia,
            font_size: 28.0,
            font_family: "Segoe UI".to_string(),
            use_custom_color: false,
            background_color: "#000000".to_string(),
            text_color: "#000000".to_string(),
            auto_scroll_speed: 3.8,
            show_page_num: true,
            show_page_file: true
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum FileOpenPreference {
    // Open the current chapter (page)
    CurrentChapter,
    // Open the next chapter (page)
    NextChapter
}

impl FileOpenPreference {
    pub fn to_usize(&self) -> usize {
        match self {
            FileOpenPreference::CurrentChapter => 0,
            FileOpenPreference::NextChapter => 1
        }
    }
}

impl fmt::Display for FileOpenPreference {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            FileOpenPreference::CurrentChapter => write!(f, "current"),
            FileOpenPreference::NextChapter => write!(f, "next")
        }
    }
}

impl FromStr for FileOpenPreference {
    type Err = ();
    
    fn from_str(input: &str) -> Result<FileOpenPreference, Self::Err> {
        match input {
            "current"  => Ok(FileOpenPreference::CurrentChapter),
            "next"  => Ok(FileOpenPreference::NextChapter),
            _      => Err(()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct FileSettings {
    pub file_open_preference: FileOpenPreference,
}

impl FileSettings {
    fn new() -> FileSettings {
        let file_open_preference = FileOpenPreference::NextChapter;
        
        FileSettings {
            file_open_preference,
        }
    }
}
