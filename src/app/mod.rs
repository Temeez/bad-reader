use gio::prelude::*;
use glib::clone;
use gtk::gdk;
use gtk::prelude::*;
use crate::app::window_state::WindowState;
use crate::ui;
use crate::appop::AppOp;
use clap::ArgMatches;
use std::path::PathBuf;
use crate::ui::Theme;
use crate::app::utils::Resources;

pub mod window_state;
pub mod error;
pub mod database;
pub mod globals;
pub mod settings;
pub mod utils;


#[derive(Clone)]
pub struct AppRuntime(glib::Sender<Box<dyn FnOnce(&mut AppOp)>>);

impl AppRuntime {
    fn init(ui: ui::UI) -> Self {
        let (app_tx, app_rx) = glib::MainContext::channel(Default::default());
        let app_runtime = Self(app_tx);
        let mut state = AppOp::new(ui, app_runtime.clone());
        
        app_rx.attach(None, move |update_state| {
            update_state(&mut state);
            
            glib::Continue(true)
        });
        
        debug!("app::AppRuntime::init");
        
        app_runtime
    }
    
    pub fn update_state_with(&self, update_fn: impl FnOnce(&mut AppOp) + 'static) {
        let _ = self.0.send(Box::new(update_fn));
    }
}

fn new(gtk_app: gtk::Application) -> AppRuntime {
    glib::set_application_name("bad-reader");
    glib::set_prgname(Some("bad-reader"));
    
    let css_file = Resources::get("css/app.css").unwrap();
    let provider = gtk::CssProvider::new();
    provider
        .load_from_data(&css_file.data)
        .expect("Cannot load app.css file");
    
    gtk::StyleContext::add_provider_for_screen(
        &gdk::Screen::default().expect("Error initializing gtk css provider."),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
    
    let mut ui = ui::UI::new(gtk_app);
    ui.init();
    
    let app_runtime = AppRuntime::init(ui);
    let window_state = WindowState::open();
    
    app_runtime.update_state_with(move |state| {
        // Do this first because `toggle_fullscreen` does visibility things.
        state.ui.main_window.show_all();
        // Load window states from the file or use defaults.
        if let Some(window_state) = window_state.unwrap() {
            state.ui.main_window.set_default_size(window_state.width, window_state.height);
            if window_state.is_fullscreen {
                state.ui.toggle_fullscreen();
            } else if window_state.is_maximized {
                state.ui.main_window.maximize();
            } else if window_state.x != 0 && window_state.y != 0 {
                state.ui.main_window.move_(window_state.x, window_state.y);
            }
        }

        
        let settings = state.settings.read();
        state.ui.set_theme(Theme::None, settings.general.theme.clone());
        state.ui.set_font_size(settings.general.font_size);
        state.ui.set_font_family(&settings.general.font_family);
        state.ui.set_custom_colors(settings.general.use_custom_color, &settings.general.background_color, &settings.general.text_color);
        
        state.ui.connect(state.app_runtime.clone());
        state.ui.settings_dialog.connect(&state.ui.builder, state.app_runtime.clone());
        state.ui.update_recent(state.app_runtime.clone(), &state.db.read().rows.clone().unwrap());

        globals::new(state);
    });
    
    debug!("app::new");
    
    app_runtime
}

pub fn on_startup(gtk_app: &gtk::Application, matches: &ArgMatches) {
    let app_runtime = new(gtk_app.clone());
    
    debug!("app::on_startup");
    
    let file = matches.value_of("file");
    let page = matches.value_of("page");
    
    let filepath = file.map(PathBuf::from);
    let num = page.map(|page| page.parse::<usize>().unwrap_or(1));
    
    gtk_app.connect_activate(clone!(@strong app_runtime => move |_| {
        let filepath = filepath.clone();
        let num = num;
        
        app_runtime.update_state_with(move |state| {
            debug!("app::on_startup -> connect_activate");
            on_activate(&state.ui);
        
            // Open the file from path if it was supplied with CLI.
            if let Some(filepath) = filepath {
                // Since left revealer is open by default, hide it when opening via CLI.
                // Simply using the `set_reveal_child()` won't work because of the slide animation
                // lag so it's half visible when the file is loading.
                state.ui.left_revealer.set_visible(false);
                // Open the file
                state.open_file_from_path(filepath, num);
            }
        });
    }));
    
    app_runtime.update_state_with(|state| {
        state.init();
    });
    
    gtk_app.connect_shutdown(move |_| {
        app_runtime.update_state_with(|state| {
            on_shutdown(state);
        });
    });
}


fn on_activate(ui: &ui::UI) {
    ui.main_window.show();
    ui.main_window.present();
    
    debug!("app::on_activate");
}

fn on_shutdown(appop: &AppOp) {
    debug!("app::on_shutdown");
    appop.quit();
}