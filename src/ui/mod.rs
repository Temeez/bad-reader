mod settings_dialog;
mod about_dialog;
mod file_chooser_dialog;
mod page_dialog;

use gtk::{prelude::*, Justification, TickCallbackId, CssProvider};

use std::path::PathBuf;
use std::fmt;
use crate::app::AppRuntime;
use select::document::Document;
use select::predicate::{Name, Any};
use crate::appop::EpubBook;
use crate::ui::settings_dialog::SettingsDialog;
use serde::{Deserialize, Serialize};
use regex::Regex;
use crate::app::database::DatabaseRow;
use crate::app::settings::Settings;
use crate::app::utils::{add_additional_style, remove_style, Resources};
use crate::PROGRAM_NAME;
use crate::app::utils::BuilderExtManualCustom;
use std::str::FromStr;
use glib::SignalHandlerId;
use std::collections::HashMap;
use std::io::Cursor;
use gdk_pixbuf::Pixbuf;

// Used when scrolling up and down with arrow keys
pub const SCROLL_AMOUNT: f64 = 120.0;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum Theme {
    None,
    Sepia,
    Dark,
    Light
}

impl fmt::Display for Theme {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Theme::None => write!(f, "none"),
            Theme::Sepia => write!(f, "sepia"),
            Theme::Dark => write!(f, "dark"),
            Theme::Light => write!(f, "light")
        }
    }
}

impl FromStr for Theme {
    type Err = ();
    
    fn from_str(input: &str) -> Result<Theme, Self::Err> {
        match input {
            "none"  => Ok(Theme::None),
            "sepia"  => Ok(Theme::Sepia),
            "dark" => Ok(Theme::Dark),
            "light" => Ok(Theme::Light),
            _      => Err(()),
        }
    }
}

pub struct UI {
    pub builder: gtk::Builder,
    pub gtk_app: gtk::Application,
    pub main_window: gtk::ApplicationWindow,
    pub menubar: gtk::MenuBar,
    pub scroller: gtk::ScrolledWindow,
    pub reader_header: gtk::TextView,
    pub reader: gtk::TextView,
    
    pub overlay: gtk::Overlay,
    pub overlay_notebook: gtk::Notebook,
    pub left_revealer: gtk::Revealer,
    pub left_content_box: gtk::Box,
    pub right_content_box: gtk::Box,
    pub spinner: gtk::Spinner,
    pub recent_handlers: HashMap<gtk::LinkButton, SignalHandlerId>,
    
    pub toc_buttons: Option<Vec<gtk::Button>>,

    pub scrolling_tick: Option<TickCallbackId>,
    pub is_fullscreen: bool,
    
    pub settings_dialog: SettingsDialog,
    
    pub additional_css: Vec<CssProvider>,
}

impl UI {
    pub fn new(gtk_app: gtk::Application) -> UI {
        let builder = gtk::Builder::from_resources("ui/reader.ui");
        builder.add_from_resources("ui/settings_dialog.ui");
        builder.add_from_resources("ui/overlay.ui");
        
        let main_window = builder.get::<gtk::ApplicationWindow>("main_window");
        let menubar = builder.get::<gtk::MenuBar>("menu_bar");
        let scroller_window = builder.get::<gtk::ScrolledWindow>("scroller_window");
        let reader_header = builder.get::<gtk::TextView>("reader_header");
        let reader = builder.get::<gtk::TextView>("reader_textview");
        
        main_window.set_application(Some(&gtk_app));
        main_window.set_title(PROGRAM_NAME);
    
        let resource = Resources::get(&"icon.png").unwrap().data;
        let icon_pix = Pixbuf::from_read(Cursor::new(resource)).expect("Cannot load pixbuf from resource.");
        main_window.set_icon(Some(&icon_pix));
    
        let settings_dialog = SettingsDialog::new(&builder, &main_window);
    
        let overlay = builder.get::<gtk::Overlay>("overlay");
        let overlay_box = builder.get::<gtk::Box>("overlay_box");
        let overlay_notebook = builder.get::<gtk::Notebook>("overlay_notebook");
    
        overlay.add_overlay(&overlay_box);
        overlay.set_overlay_pass_through(&overlay_box, true);
        
        let spinner = gtk::Spinner::new();
        overlay.add_overlay(&spinner);
        overlay.set_overlay_pass_through(&spinner, true);
        
        let left_revealer = builder.get::<gtk::Revealer>("left_revealer");
        let left_content_box = builder.get::<gtk::Box>("left_content_box");
        let right_content_box = builder.get::<gtk::Box>("right_content_box");
        
        UI {
            builder,
            gtk_app,
            main_window,
            menubar,
            scroller: scroller_window,
            reader_header,
            reader,
            
            overlay,
            overlay_notebook,
            left_revealer,
            left_content_box,
            right_content_box,
            spinner,
            recent_handlers: HashMap::new(),
            
            toc_buttons: None,
            scrolling_tick: None,
            is_fullscreen: false,
            
            settings_dialog,
            
            additional_css: vec![]
        }
    }
    
    pub fn init(&mut self) {
        debug!("UI init doned");
    }
    
    pub fn connect(&self, app_runtime: AppRuntime) {
        self.main_window.connect_button_release_event(glib::clone!(@strong app_runtime => move |_, event| {
            match event.button() {
                // MOUSE BUTTON 4 (previous)
                4 => {
                    app_runtime.update_state_with(move |state| {
                        state.previous_page();
                    });
                },
                // MOUSE BUTTON 5 (next)
                5 => {
                    app_runtime.update_state_with(move |state| {
                        state.next_page();
                    });
                }
                _ => {}
            }
            
            gtk::Inhibit(false)
        }));
    
        self.main_window.connect_key_press_event(glib::clone!(@strong app_runtime => move |_window, event| {
            match event.keycode() {
                // ARROW UP
                Some(38) => {
                    app_runtime.update_state_with(|state| {
                        state.ui.scroll_up(SCROLL_AMOUNT);
                    });
                },
                // ARROW DOWN
                Some(40) => {
                    app_runtime.update_state_with(|state| {
                        state.ui.scroll_down(SCROLL_AMOUNT);
                    });
                },
                // ARROW LEFT
                Some(37) => {
                    app_runtime.update_state_with(move |state| {
                        state.previous_page();
                    });
                },
                // ARROW RIGHT
                Some(39) => {
                    app_runtime.update_state_with(move |state| {
                        state.next_page();
                    });
                }
                _ => {}
            }
        
            gtk::Inhibit(false)
        }));
    
        // Change the reader font size on the fly
        let fontsize = &self.builder.get::<gtk::SpinButton>( "fontsize_spin");
        fontsize.connect_changed(glib::clone!(@strong app_runtime => move |elem| {
            let fontsize = elem.value();
            app_runtime.update_state_with(move |state| {
                state.ui.set_font_size(fontsize);
            });
        }));
    }
    
    // Open settings dialog
    pub fn show_settings_dialog(&self, settings: Settings) {
        self.settings_dialog.update(&self.builder, &settings);
        self.settings_dialog.dialog.show();
    }
    
    /// Update the box which contains buttons for opening pages (left side).
    pub fn update_toc(&self, app_runtime: AppRuntime, book: &mut EpubBook) {
        // Clear the whole box before adding new elements
        for child in self.left_content_box.children() {
            self.left_content_box.remove(&child);
        }
        
        // Get the spine from the epub doc and make the buttons
        for spine_item in &book.doc.spine {
            let ch = gtk::LinkButton::new(spine_item);
            let new_chap = book
                .doc
                .resource_uri_to_chapter(&book
                    .doc
                    .resources
                    .get(spine_item)
                    .unwrap()
                    .0
                );
            ch.set_label(&book.current_chapter_file_name(spine_item));
            ch.connect_activate_link(glib::clone!(@strong app_runtime => move |_| {
                app_runtime.update_state_with(move |state| {
                    // Open the requested page
                    state.open_page_send(new_chap.unwrap());
                    // Close the toc revealer
                    state.ui.toggle_toc();
                });
                // Inhibit because the link is not a real link
                gtk::Inhibit(true)
            }));
            self.left_content_box.add(&ch);
        }
    
        self.left_content_box.show_all();
    }
    
    /// Update the box which contains recent files (right side).
    ///
    /// TODO: Probably inefficient, improve.
    pub fn update_recent(&mut self, app_runtime: AppRuntime, rows: &[DatabaseRow]) {
        // If there are old handlers then disconnect them
        // so they can be added back.
        if !self.recent_handlers.is_empty() {
            for (btn, handler) in self.recent_handlers.drain() {
                btn.disconnect(handler);
            }
        }
    
        // Remove any child elements
        for child in self.right_content_box.children() {
            self.right_content_box.remove(&child);
        }
        
        for row in rows {
            let link_button = gtk::LinkButton::new(row.file.to_str().unwrap());
            link_button.set_label(&format!("{} — page {}", row.filename.to_str().unwrap(), row.current_page + 1));
            let handler = link_button.connect_activate_link(glib::clone!(@strong app_runtime => move |btn| {
                let file_path = PathBuf::from(btn.uri().unwrap().to_string());
                app_runtime.update_state_with(move |state| {
                    // Open the file
                    state.open_file_from_path(file_path, None);
                    // Close the recent revealer
                    // state.ui.toggle_recent_files();
                });
                gtk::Inhibit(true)
            }));
            self.right_content_box.add(&link_button);
            self.recent_handlers.insert(link_button, handler);
        }
        
        self.right_content_box.show_all();
    }
    
    pub fn update(&self, book: &mut EpubBook, settings: &Settings) {
        let chapter_list_label = &self.builder.get::<gtk::Label>("chapter_list_label");
        
        let current_chapter_id = book.doc.get_current_id().unwrap();
        let current_chapter_filename = book.current_chapter_file_name(&current_chapter_id);
        chapter_list_label.set_label(&format!("Reading: {:?}", current_chapter_filename));
        
        if !settings.general.show_page_num && !settings.general.show_page_file {
            self.reader_header.set_visible(false);
        } else {
            self.reader_header.set_visible(true);
        }
        
        let mut header_content = String::new();
        if settings.general.show_page_num && !settings.general.show_page_file {
            header_content.push_str(&format!("Page {}", book.doc.get_current_page() + 1));
        } else if settings.general.show_page_file && !settings.general.show_page_num {
            header_content.push_str(&current_chapter_filename);
        } else {
            header_content.push_str(&format!("Page {} - {}", book.doc.get_current_page() + 1, current_chapter_filename));
        }
        
        self.reader_header
            .buffer()
            .expect("Could not get buffer")
            .set_text(&header_content);
        
        let book_content = match book.doc.get_current_str() {
            Ok(content) => content,
            Err(e) => {
                error!("{}", e);
                
                "Error loading content :(".to_string()
            }
        };
        
        let book_filename = book.file.file_name().unwrap().to_str().unwrap().to_string();
        
        self.main_window.set_title(&format!("{} - {} - {}", current_chapter_filename, book_filename, PROGRAM_NAME));
        
        let mut content = String::new();
        
        if book_content.contains("</p>") {
            let html = Document::from(book_content.as_str());
            for tag in html.select(Any) {
                if let Some(name) = tag.name() {
                    if name == "p" {
                        content.push_str(tag.text().as_str());
                        content.push_str("\n\n");
                    } else {
                        // content.push_str("");
                    }
                }
            }
        } else {
            let re = Regex::new(r"<br.?/>").unwrap();
            let result = re.replace_all(book_content.as_str(), "\n");
    
            let html = Document::from(result.to_string().as_str());
            for body in html.select(Name("body")) {
                content.push_str(body.text().as_str());
            }
        }
        
        self.reader.set_justification(Justification::Left);
        self.reader
            .buffer()
            .expect("Could not get buffer")
            .set_text(&content);
    }
    
    /// Removes the old theme class from elements and
    /// adds the new theme class.
    pub fn set_theme(&mut self, old_theme: Theme, theme: Theme) {
        debug!("Set theme: {:?} | old: {:?}", theme, old_theme);
        // Do nothing if theme didn't change
        if old_theme == theme {
            return;
        }
        
        let old_theme_name = old_theme.to_string();
        let theme_name = theme.to_string();
    
        let recent_scroller = &self.builder.get::<gtk::ScrolledWindow>("recent_scroller");
        let toc_scroller = &self.builder.get::<gtk::ScrolledWindow>("toc_scroller");
        let toc_container = &self.builder.get::<gtk::Box>("toc_container");
        
        self.reader_header.style_context().remove_class(&old_theme_name);
        self.reader_header.style_context().add_class(&theme_name);
        
        self.reader.style_context().remove_class(&old_theme_name);
        self.reader.style_context().add_class(&theme_name);
        
        self.scroller.style_context().remove_class(&old_theme_name);
        self.scroller.style_context().add_class(&theme_name);
        
        self.overlay_notebook.style_context().remove_class(&old_theme_name);
        self.overlay_notebook.style_context().add_class(&theme_name);
    
        recent_scroller.style_context().remove_class(&old_theme_name);
        recent_scroller.style_context().add_class(&theme_name);
        
        toc_scroller.style_context().remove_class(&old_theme_name);
        toc_scroller.style_context().add_class(&theme_name);
        
        toc_container.style_context().remove_class(&old_theme_name);
        toc_container.style_context().add_class(&theme_name);
    }
    
    /// Set font size for the reader element in pixels.
    pub fn set_font_size(&mut self, font_size: f64) {
        let css_string = format!("textview {{ font-size: {}px; }}", font_size);
    
        add_additional_style(css_string);
    }
    
    /// Set the font family for the reader element.
    pub fn set_font_family(&mut self, font_family: &str) {
        let css_string = format!("textview {{ font-family: {}; }}", font_family);
    
        add_additional_style(css_string);
    }
    
    /// Set custom background and text color for the reader.
    pub fn set_custom_colors(&mut self, use_color: bool, bg: &str, text: &str) {
        // Always delete the old provider.
        if let Some(prov) = self.additional_css.pop() {
            remove_style(prov);
        }
        
        // If using a custom color then set it and save the provider to a vector
        // so it can be deleted later.
        if use_color {
            let mut css_string = String::new();
            if !bg.is_empty() {
                css_string.push_str(&format!("\
                    .reader-scroller {{ background-color: {}; }}\
                     textview text {{ background-color: {}; }}\
                ", bg, bg));
            }
            if !text.is_empty() {
                css_string.push_str(&format!("\
                    textview text {{ color: {}; }}\
                ", text));
            }
            let prov = add_additional_style(css_string);
            self.additional_css.push(prov);
        }
    }
    
    /// Toggle fullscreen mode on or off. Hides the menubar on fullscreen.
    pub fn toggle_fullscreen(&mut self) {
        if self.is_fullscreen {
            self.unfullscreen();
        } else {
            self.fullscreen();
        }
    
        // Hide menu bar when in fullscreen
        self.menubar.set_visible(!self.is_fullscreen);
    }
    
    /// Set fullscreen mode on
    fn fullscreen(&mut self) {
        self.main_window.fullscreen();
        self.is_fullscreen = true;
    }
    
    /// Set fullscreen mode off
    fn unfullscreen(&mut self) {
        self.main_window.unfullscreen();
        self.is_fullscreen = false;
    }
    
    /// Start scrolling down and save the `TickCallbackId` so it can be stopped.
    pub fn start_scrolling_down(&mut self, speed: f64) {
        self.scrolling_tick = scroll_down_automatic(&mut self.scroller, speed);
    }
    
    /// Stop scrolling down.
    pub fn stop_scrolling_down(&mut self) {
        if let Some(tick_id) = self.scrolling_tick.take() {
            tick_id.remove();
        }
    }
    
    /// Toggle scrolling down on or off.
    pub fn toggle_scrolling(&mut self, speed: f64) {
        if self.scrolling_tick.is_some() {
            debug!("Scrolling stop!");
            self.stop_scrolling_down();
        } else {
            debug!("Scrolling start!");
            self.start_scrolling_down(speed);
        }
    }
    
    /// Scroll to the top.
    pub fn scroll_to_bottom(&mut self) {
        let adj = self.scroller.vadjustment();
        adj.set_value(adj.upper() - adj.page_size());
    }
    
    /// Scroll to the bottom.
    pub fn scroll_to_top(&mut self) {
        let adj = self.scroller.vadjustment();
        adj.set_value(0.0);
    }
    
    /// Scroll down by a specified amount.
    pub fn scroll_down(&mut self, amount: f64) {
        let adj = self.scroller.vadjustment();
        adj.set_value(adj.value() + amount);
    }
    
    /// Scroll up by a specified amount.
    pub fn scroll_up(&mut self, amount: f64) {
        let adj = self.scroller.vadjustment();
        adj.set_value(adj.value() - amount);
    }

    /// Toggle recent files revealer and notebook on or off.
    pub fn toggle_recent_files(&mut self) {
        // Check if the revealer needs to be opened or closed
        if !(self.left_revealer.is_child_revealed() && self.overlay_notebook.page() == 1) {
            self.left_revealer.set_reveal_child(!self.left_revealer.is_child_revealed());
        }
        // Open the correct notebook page
        self.overlay_notebook.set_page(0);
    }
    
    /// Toggle table of contents revealer and notebook on or off.
    pub fn toggle_toc(&mut self) {
        // Check if the revealer needs to be opened or closed
        if !(self.left_revealer.is_child_revealed() && self.overlay_notebook.page() == 0) {
            self.left_revealer.set_reveal_child(!self.left_revealer.is_child_revealed());
        }
        // Open the correct notebook page
        self.overlay_notebook.set_page(1);
    }
    
    /// Toggle spinner to show that the app is doing something.
    pub fn toggle_spinner(&mut self, value: bool) {
        self.spinner.set_visible(value);
        if value {
            self.spinner.start();
        } else {
            self.spinner.stop();
        }
    }
}

/// Automatic scrolling down logic.
fn scroll_down_automatic(view: &mut gtk::ScrolledWindow, duration_multiplier: f64) -> Option<TickCallbackId> {
    let adj = view.vadjustment();
    if let Some(clock) = view.frame_clock() {
        let start = adj.value();
        let real_end = adj.upper() - adj.page_size();
        // Large default end value so the length doesn't
        // affect the scrolling speedoverlay_box
        let mut end = 99_999.0;
        // Fallback just in case. I don't expect epub pages to past 20_000 normally.
        if real_end > end {
            end = real_end
        };
        // Larger value for slower scrolling
        let duration = (duration_multiplier * 1_000_000.0) as i64;
        
        let start_time = clock.frame_time();
        let end_time = start_time + 1000 * duration;
        let tick_id = view.add_tick_callback(move |_view, clock| {
            let now = clock.frame_time();
            if now < end_time && (adj.value() - end).abs() > f64::EPSILON {
                let t = (now - start_time) as f64 / (end_time - start_time) as f64;
                // t = ease_out_cubic(t);
                // Comment out start to make the speed constant regardless of
                // the current scroll position
                // Otherwise it gets slower the closer the end is
                // adj.set_value(start + t * (end - start));
                adj.set_value(start + t * end);
                glib::Continue(true)
            } else {
                adj.set_value(end);
                glib::Continue(false)
            }
        });
        
        return Some(tick_id)
    }
    
    None
}

// /* From clutter-easing.c, based on Robert Penner's
//  * infamous easing equations, MIT license.
//  */
// fn ease_out_cubic (t: f64) -> f64 {
//     let p = t - 1f64;
//     return p * p * p + 1f64;
// }
