use crate::appop::AppOp;
use gtk::prelude::{ComboBoxExt, SpinButtonExt, ToggleButtonExt, EntryExt};
use crate::app::settings::FileOpenPreference;
use std::sync::Arc;
use parking_lot::lock_api::RwLock;
use crate::ui::{Theme};
use crate::app::utils::BuilderExtManualCustom;
use std::str::FromStr;


impl AppOp {
    /// Used when settings dialog sends a OK response.
    pub fn update_settings(&mut self) {
        debug!("appop::update_settings");
        let builder = &self.ui.builder;
        
        let theme_selector = builder.get::<gtk::ComboBoxText>( "theme_selector");
        let auto_scroll_speed = builder.get::<gtk::SpinButton>( "auto_scroll_speed");
        let show_page_num = builder.get::<gtk::CheckButton>( "show_page_num");
        let show_page_file = builder.get::<gtk::CheckButton>( "show_page_file");
        let fontsize = builder.get::<gtk::SpinButton>( "fontsize_spin");
        let fontfamily = builder.get::<gtk::Entry>( "fontfamily_entry");
        let use_custom_color = builder.get::<gtk::CheckButton>( "use_custom_color");
        let background_color = builder.get::<gtk::Entry>( "background_color");
        let text_color = builder.get::<gtk::Entry>( "text_color");
        
        let file_open_preference_combobox = builder.get::<gtk::ComboBoxText>( "file_open_preference_combobox");
        
        let mut new_settings = self.settings.write().clone();
        new_settings.general.theme = Theme::from_str(theme_selector.active_id().unwrap().as_str()).unwrap();
    
        new_settings.general.auto_scroll_speed = auto_scroll_speed.value();
        new_settings.general.show_page_num = show_page_num.is_active();
        new_settings.general.show_page_file = show_page_file.is_active();
        new_settings.general.font_size = fontsize.value();
        new_settings.general.font_family = fontfamily.text().to_string();
        new_settings.general.use_custom_color = use_custom_color.is_active();
        new_settings.general.background_color = background_color.text().to_string();
        new_settings.general.text_color = text_color.text().to_string();
        
        new_settings.file.file_open_preference = FileOpenPreference::from_str(file_open_preference_combobox.active_id().unwrap().as_str()).unwrap();
        
        self.app_runtime.update_state_with(move |state| {
            state.ui.set_theme(state.settings.read().general.theme.clone(), new_settings.general.theme.clone());
            state.ui.set_font_family(&new_settings.general.font_family);
            state.ui.set_custom_colors(new_settings.general.use_custom_color, &new_settings.general.background_color, &new_settings.general.text_color);
            
            // Update the reader header if a book is loaded
            if let Some(book) = state.epub_book.as_mut() {
                state.ui.update(book, &new_settings);
            }
    
            // Save new settings to file and app
            new_settings.write().expect("Cannot write to the settings file.");
            state.settings = Arc::new(RwLock::new(new_settings));
        });
    }
}