use gtk::{Dialog, ResponseType};
use gtk::prelude::{WidgetExtManual};
use gtk::{prelude::*};
use crate::app::AppRuntime;
use crate::app::settings::{Settings};
use crate::PROGRAM_NAME;
use crate::app::utils::BuilderExtManualCustom;


#[derive(Clone, Debug)]
pub struct SettingsDialog {
    pub dialog: Dialog,
}

impl SettingsDialog {
    pub fn new(builder: &gtk::Builder, parent: &gtk::ApplicationWindow) -> SettingsDialog {
        let dialog = cascade! {
            builder.get::<gtk::Dialog>("settings_dialog");
            ..set_title(&format!("Settings - {}", PROGRAM_NAME));
            ..set_modal(true);
            ..set_transient_for(Some(parent));
            ..connect_response(|dialog, response_type| {
                match response_type {
                    gtk::ResponseType::Apply => dialog.hide(),
                    gtk::ResponseType::Cancel => dialog.hide(),
                    _ => ()
                }
            });
        };
        
        let _ = cascade! {
            builder.get::<gtk::ListBoxRow>("application_settings_listboxrow");
            ..activate();
        };
        
        SettingsDialog {
            dialog
        }
    }
    
    pub fn connect(&self, builder: &gtk::Builder, app_runtime: AppRuntime) {
        let settings_notebook = builder.get::<gtk::Notebook>("settings_notebook");
        let settings_listbox = builder.get::<gtk::ListBox>("settings_listbox");
        
        // Hide the element instead of deleting it when the close button is clicked
        self.dialog.connect_delete_event(move |dialog, _event| {
            dialog.hide_on_delete()
        });
        
        settings_listbox.connect_row_activated(move |_list, row| {
            settings_notebook.set_current_page(Some(row.index() as u32));
        });
        
        self.dialog.connect_key_release_event(glib::clone!(@strong app_runtime => move |dialog, event| {
            match event.keycode() {
                Some(13) => {
                    dialog.response(ResponseType::Ok);
                },
                Some(27) => {
                    dialog.response(ResponseType::Cancel);
                },
                _ => {}
            }
            
            gtk::Inhibit(false)
        }));
        self.dialog.connect_response(glib::clone!(@strong app_runtime => move |dialog, response_type| {
            match response_type {
                gtk::ResponseType::Ok => {
                    dialog.hide();
                    
                    app_runtime.update_state_with(move |state| {
                        state.update_settings();
                    });
                },
                gtk::ResponseType::Cancel => dialog.hide(),
                _ => ()
            }
        }));
    }
    
    pub fn update(&self, builder: &gtk::Builder, settings: &Settings) {
        let theme_selector = builder.get::<gtk::ComboBoxText>("theme_selector");
        let auto_scroll_speed = builder.get::<gtk::SpinButton>("auto_scroll_speed");
        let show_page_num = builder.get::<gtk::CheckButton>("show_page_num");
        let show_page_file = builder.get::<gtk::CheckButton>("show_page_file");

        let fontsize = builder.get::<gtk::SpinButton>("fontsize_spin");
        let fontfamily = builder.get::<gtk::Entry>("fontfamily_entry");
        let use_custom_color = builder.get::<gtk::CheckButton>( "use_custom_color");
        let background_color = builder.get::<gtk::Entry>( "background_color");
        let text_color = builder.get::<gtk::Entry>( "text_color");

        theme_selector.set_active_id(Some(settings.general.theme.to_string().as_str()));
        auto_scroll_speed.set_value(settings.general.auto_scroll_speed);
        show_page_num.set_active(settings.general.show_page_num);
        show_page_file.set_active(settings.general.show_page_file);
        fontsize.set_value(settings.general.font_size);
        fontfamily.set_text(&settings.general.font_family);
        use_custom_color.set_active(settings.general.use_custom_color);
        background_color.set_text(&settings.general.background_color);
        text_color.set_text(&settings.general.text_color);
    
        let _ = cascade! {
            builder.get::<gtk::ComboBoxText>("file_open_preference_combobox");
            ..set_active_id(Some(settings.file.file_open_preference.to_string().as_str()));
        };
    }
}