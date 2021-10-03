use glib::{IsA, Object};
use gtk::prelude::{BuilderExt, BuilderExtManual, CssProviderExt};
use gtk::{CssProvider, Builder};
use std::env::current_exe;
use std::path::PathBuf;
use rust_embed::RustEmbed;


#[derive(RustEmbed)]
#[folder = "resources/"]
pub struct Resources;

/// Helper trait & method to make it a bit easier to get the object from ui file.
///
/// Before:
/// ```
/// let button = builder
///     .object::<gtk::Button>("button")
///     .expect("Cannot find button in ui file.");
/// ```
/// After:
/// ```
/// let button = builder.get::<gtk::Button>("button");
/// ```
pub trait BuilderExtManualCustom {
    fn get<T: IsA<Object>>(&self, name: &str) -> T;
    
    fn from_resources(file: &str) -> Builder;
    
    fn add_from_resources(&self, file: &str);
}

impl<O: IsA<Builder>> BuilderExtManualCustom for O {
    fn get<T: IsA<Object>>(&self, name: &str) -> T {
        self
            .object::<T>(name)
            .unwrap_or_else(|| panic!("Cannot find {} in ui file.", name))
    }
    
    fn from_resources(file: &str) -> Builder {
        let resource = String::from_utf8(Resources::get(file).unwrap().data.as_ref().to_vec()).unwrap();
        
        Builder::from_string(&resource)
    }
    
    fn add_from_resources(&self, file: &str) {
        let resource = String::from_utf8(Resources::get(file).unwrap().data.as_ref().to_vec()).unwrap();
        
        self.add_from_string(&resource).unwrap_or_else(|_| panic!("Cannot add {} from string", file));
    }
}

pub fn add_additional_style(css_string: String) -> CssProvider {
    let provider = cascade! {
        gtk::CssProvider::new();
        ..load_from_data(&css_string.into_bytes()).expect("Cannot load css from data.");
    };
    
    gtk::StyleContext::add_provider_for_screen(
        &gdk::Screen::default().expect("Cannot get default screen."),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
    
    provider
}

pub fn remove_style(provider: CssProvider) {
    gtk::StyleContext::remove_provider_for_screen(
        &gdk::Screen::default().expect("Cannot get default screen."),
        &provider
    );
}

pub fn working_dir(file: &str) -> String {
    #[cfg(debug_assertions)]
    let dir = match current_exe() {
        Ok(mut path) => {
            debug!("current_exe: {:?}", path);
            path.pop();
            path.pop();
            path.pop();
            path
        },
        Err(e) => {
            error!("Could not get current_exe path. {:?}", e);
            PathBuf::new()
        }
    };
    
    #[cfg(not(debug_assertions))]
        let dir = match current_exe() {
        Ok(mut path) => {
            debug!("current_exe: {:?}", path);
            path.pop();
            path
        },
        Err(e) => {
            error!("Could not get current_exe path. {:?}", e);
            PathBuf::new()
        }
    };
    
    if let Some(filepath) = dir.join(file).to_str() {
        return filepath.to_string()
    }
    
    "".to_string()
}