use super::UI;

use gtk::prelude::*;
use gtk::{FileChooserAction, ResponseType};
use crate::app::AppRuntime;


impl UI {
    pub fn file_chooser_dialog(&self, app_runtime: AppRuntime) {
        let dialog = cascade! {
            gtk::FileChooserDialog::new(Some("Choose epub file to open"), Some(&self.main_window), FileChooserAction::Open);
            ..set_modal(true);
            ..set_transient_for(Some(&self.main_window));
            ..add_buttons(&[
                ("Open", ResponseType::Ok),
                ("Cancel", ResponseType::Cancel),
            ]);
        };
        
        dialog.connect_response(glib::clone!(@strong app_runtime => move |dialog, response| {
            match response {
                ResponseType::Ok => {
                    if let Some(filename) = dialog.filename() {
                        app_runtime.update_state_with(move |state| {
                            state.open_file_from_path(filename, None);
                            state.ui.update_recent(state.app_runtime.clone(), &state.db.read().clone().rows.unwrap());
                        });
                        dialog.close();
                    }
                },
                ResponseType::Cancel => {
                    dialog.close();
                },
                _ => {}
            }
        }));
        
        dialog.show_all();
        
        // Close the left revealer if it's open.
        self.left_revealer.set_reveal_child(false);
    }
}