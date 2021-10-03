use super::UI;

use gtk::prelude::*;
use crate::{PROGRAM_NAME, VERSION, ABOUT};


impl UI {
    pub fn about_dialog(&self) {
        let dialog = cascade! {
            gtk::AboutDialog::new();
            ..set_comments(Some(ABOUT));
            ..set_modal(true);
            ..set_version(Some(VERSION));
            ..set_program_name(PROGRAM_NAME);
            ..set_transient_for(Some(&self.main_window));
            ..connect_response(move |d, _| {
                d.close();
            });
        };
        
        dialog.show();
    }
}