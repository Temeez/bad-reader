use gtk::prelude::*;

use crate::app::error::SettingsError;
use std::path::{Path};
use std::io::{BufWriter, BufReader};
use std::fs::File;
use anyhow::{Context};
use bincode::{deserialize_from, serialize_into};
use serde::{Deserialize, Serialize};
use crate::app::utils::working_dir;


static CONFIG_NAME: &str = "bad-reader.state";

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct WindowState {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub is_maximized: bool,
    pub is_fullscreen: bool,
}

impl Default for WindowState {
    fn default() -> WindowState {
        WindowState {
            x: 300,
            y: 300,
            width: 860,
            height: 600,
            is_maximized: false,
            is_fullscreen: false
        }
    }
}

impl WindowState {
    pub fn from_window(window: &gtk::ApplicationWindow, is_fullscreen: bool) -> WindowState {
        let is_maximized = window.is_maximized();
        let position = window.position();
        let size = window.size();
        let x = position.0;
        let y = position.1;
        let width = size.0;
        let height = size.1;
        
        WindowState {
            x,
            y,
            width,
            height,
            is_maximized,
            is_fullscreen,
        }
    }
    
    pub fn write(&self) -> Result<(), SettingsError> {
        let path = Path::new(&working_dir(CONFIG_NAME)).to_path_buf();
        let mut f = BufWriter::new(File::create(&path).unwrap());
        serialize_into(&mut f, self)
            .context(SettingsError::WriteToDisk)?;
        Ok(())
    }
    
    pub fn open() -> Result<Option<Self>, SettingsError> {
        let path = Path::new(&working_dir(CONFIG_NAME)).to_path_buf();
        if path.as_path().exists() {
            let f = File::open(&path)
                .context(SettingsError::ReadFromDisk)?;
            if let Ok(settings) = deserialize_from(BufReader::new(&f))
                .context(SettingsError::Unknown) {
                return Ok(Some(settings));
            }
        }
        
        Ok(None)
    }
}
