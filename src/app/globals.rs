use gio::prelude::*;
use gio::SimpleAction;

use crate::appop::AppOp;
use gtk::prelude::{GtkApplicationExt, WidgetExt, GtkWindowExt};


pub fn new(appop: &AppOp) {
    let app = &appop.ui.gtk_app;
    let app_runtime = appop.app_runtime.clone();
    
    let toggle_fullscreen = SimpleAction::new("toggle_fullscreen", None);
    let toggle_toc = SimpleAction::new("toggle_toc", None);
    let toggle_recent = SimpleAction::new("toggle_recent", None);
    let settings = SimpleAction::new("open_settings", None);
    let about = SimpleAction::new("open_about", None);
    let quit = SimpleAction::new("quit", None);
    let auto_scroll = SimpleAction::new("auto_scroll", None);
    let file_chooser = SimpleAction::new("open_file_chooser", None);
    let scroll_down = SimpleAction::new("scroll_down", None);
    let scroll_up = SimpleAction::new("scroll_up", None);
    let scroll_down_amount = SimpleAction::new("scroll_down_amount", None);
    let scroll_up_amount = SimpleAction::new("scroll_up_amount", None);
    let page_manual = SimpleAction::new("page", None);
    
    app.add_action(&toggle_fullscreen);
    app.add_action(&toggle_toc);
    app.add_action(&toggle_recent);
    app.add_action(&settings);
    app.add_action(&about);
    app.add_action(&quit);
    app.add_action(&auto_scroll);
    app.add_action(&file_chooser);
    app.add_action(&scroll_down);
    app.add_action(&scroll_up);
    app.add_action(&scroll_down_amount);
    app.add_action(&scroll_up_amount);
    app.add_action(&page_manual);
    
    // Some of these accels are set in the menubar via glade but during the fullscreen
    // when the menubar is hidden they don't work and they don't seem to get duplicated so
    // this is fine.
    app.set_accels_for_action("app.toggle_fullscreen", &["F11", "F"]);
    app.set_accels_for_action("app.toggle_toc", &["C"]);
    app.set_accels_for_action("app.toggle_recent", &["R"]);
    app.set_accels_for_action("app.open_about", &["F1"]);
    app.set_accels_for_action("app.open_settings", &["F2"]);
    app.set_accels_for_action("app.quit", &["<Primary>Q"]);
    app.set_accels_for_action("app.auto_scroll", &["section", "F8"]);
    app.set_accels_for_action("app.open_file_chooser", &["<Primary>O"]);
    app.set_accels_for_action("app.scroll_down", &["End"]);
    app.set_accels_for_action("app.scroll_up", &["Home"]);
    app.set_accels_for_action("app.scroll_down_amount", &["Page_Down", "space"]);
    app.set_accels_for_action("app.scroll_up_amount", &["Page_Up"]);
    app.set_accels_for_action("app.page", &["G"]);
    
    
    auto_scroll.connect_activate(glib::clone!(@strong app_runtime => move |_, _| {
        app_runtime.update_state_with(|state| {
            state.ui.toggle_scrolling(state.settings.read().general.auto_scroll_speed);
        });
    }));
    
    scroll_down.connect_activate(glib::clone!(@strong app_runtime => move |_, _| {
        app_runtime.update_state_with(|state| {
            state.ui.scroll_to_bottom();
        });
    }));
    
    scroll_up.connect_activate(glib::clone!(@strong app_runtime => move |_, _| {
        app_runtime.update_state_with(|state| {
            state.ui.scroll_to_top();
        });
    }));
    
    scroll_down_amount.connect_activate(glib::clone!(@strong app_runtime => move |_, _| {
        app_runtime.update_state_with(|state| {
            let height = state.ui.main_window.size().1 as f64;
            state.ui.scroll_down(height * 0.8);
        });
    }));
    
    scroll_up_amount.connect_activate(glib::clone!(@strong app_runtime => move |_, _| {
        app_runtime.update_state_with(|state| {
            let height = state.ui.main_window.size().1 as f64;
            state.ui.scroll_up(height * 0.8);
        });
    }));
    
    quit.connect_activate(glib::clone!(@strong app_runtime => move |_action, _param| {
        app_runtime.update_state_with(|state| {
            state.ui.main_window.close();
        });
    }));
    
    toggle_fullscreen.connect_activate(glib::clone!(@strong app_runtime => move |_, _| {
        app_runtime.update_state_with(|state| {
            state.ui.toggle_fullscreen();
        });
    }));
    
    toggle_toc.connect_activate(glib::clone!(@strong app_runtime => move |_, _| {
        app_runtime.update_state_with(|state| {
            state.ui.toggle_toc();
        });
    }));
    
    toggle_recent.connect_activate(glib::clone!(@strong app_runtime => move |_, _| {
        app_runtime.update_state_with(|state| {
            state.ui.toggle_recent_files();
        });
    }));
    
    about.connect_activate(glib::clone!(@strong app_runtime => move |_, _| {
        app_runtime.update_state_with(|state| state.ui.about_dialog());
    }));
    
    page_manual.connect_activate(glib::clone!(@strong app_runtime => move |_, _| {
        app_runtime.update_state_with(|state| state.toggle_page_dialog());
    }));
    
    settings.connect_activate(glib::clone!(@strong app_runtime => move |_, _| {
        app_runtime.update_state_with(|state| {
            state.ui.show_settings_dialog(state.settings.read().clone());
        });
    }));
    
    file_chooser.connect_activate(glib::clone!(@strong app_runtime => move |_, _| {
        app_runtime.update_state_with(|state| state.ui.file_chooser_dialog(state.app_runtime.clone()));
    }));
    
    // Keep track of the fullscren state.
    appop.ui.main_window.connect_window_state_event(glib::clone!(@strong app_runtime => move |_window, event| {
        let is_fullscreen = event.new_window_state().contains(gdk::WindowState::FULLSCREEN);
        app_runtime.update_state_with(move |state| {
            state.ui.is_fullscreen = is_fullscreen;
        });
        
        gtk::Inhibit(false)
    }));
}