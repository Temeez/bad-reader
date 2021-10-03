use super::UI;

use gtk::prelude::{DialogExt, WidgetExt, GtkWindowExt, SpinButtonExt, BoxExt};
use gtk::{ResponseType};
use crate::PROGRAM_NAME;
use crate::app::AppRuntime;


impl UI {
    pub fn page_dialog(&self, app_runtime: AppRuntime, current_page: usize, last_page: usize) {
        let label = gtk::Label::new(Some("Goto page"));
        
        let spin_btn = cascade! {
            gtk::SpinButton::with_range(1.0, last_page as f64, 1.0);
            ..set_value((current_page + 1) as f64);
            ..set_margin(10);
            ..set_margin_top(0);
        };
        
        let _ = cascade! {
            gtk::Dialog::new();
            ..set_title(&format!("Open page - {}", PROGRAM_NAME));
            ..set_modal(true);
            ..set_transient_for(Some(&self.main_window));
            ..set_decorated(false);
            ..add_button("Ok", ResponseType::Ok);
            ..add_button("Cancel", ResponseType::Cancel);
            ..content_area().pack_start(&label, true, false, 12);
            ..content_area().pack_end(&spin_btn, true, false, 12);
            ..set_size_request(300, 140);
            ..connect_key_press_event(move |dialog, event| {
                // ENTER
                if let Some(13) = event.keycode() {
                    dialog.response(ResponseType::Ok);
                }
                gtk::Inhibit(false)
            });
            ..connect_response(glib::clone!(@strong app_runtime => move |dialog, response| {
                spin_btn.update();
                let mut num = spin_btn.value_as_int() as usize;
                // If num somehow ends up as 0 then make it 1 so there won't be
                // overflow error when trying to substract 1.
                if num == 0 {
                    num = 1;
                }
                if response == ResponseType::Ok {
                    app_runtime.update_state_with(move |state| {
                        state.open_page_send(num - 1);
                    });
                }
                dialog.close();
            }));
            ..show_all();
        };
    }
}