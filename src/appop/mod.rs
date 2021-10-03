pub mod messages;
pub mod settings;

use std::sync::Arc;

use gtk::prelude::*;
use parking_lot::RwLock;

use crate::app::{AppRuntime};
use crate::ui;
use std::path::{PathBuf, Path};
use epub::doc::EpubDoc;
use std::io::BufReader;
use std::fs::File;
use std::ffi::{OsStr, OsString};
use crate::app::database::{Database, read_database, DatabaseRow};
use std::thread;
use core::mem;
use crate::app::settings::Settings;
use crate::app::window_state::WindowState;


pub struct EpubBook {
    pub file: PathBuf,
    pub doc: EpubDoc<BufReader<File>>,
    pub initial_page: Option<usize>,
}

impl EpubBook {
    /// Prettify current chapter file name. E.g: `EPUB/chapter_1.xml` -> `chapter_1.xml`
    pub fn current_chapter_file_name(&self, current_chapter_id: &str) -> String {
        // let current_chapter_id = self.doc.get_current_id().unwrap();
        let current_chapter_file = self.doc.resources.get(current_chapter_id).unwrap().0.to_str().unwrap();
        
        current_chapter_file
            .replace("EPUB/", "")
            .replace("OEBPS/", "")
    }
}

pub struct AppOp {
    pub app_runtime: AppRuntime,
    pub ui: ui::UI,
    pub settings: Arc<RwLock<Settings>>,
    pub db: Arc<RwLock<Database>>,
    
    pub epub_book: Option<EpubBook>,
    
    pub open_page_sender: Option<glib::Sender<usize>>,
    pub open_epub_book_sender: Option<glib::Sender<EpubBook>>,
}

impl AppOp {
    pub fn new(ui: ui::UI, app_runtime: AppRuntime) -> AppOp {
        debug!("appop::new");
        
        let settings = Arc::new(
            RwLock::new(
                Settings::open().expect("Failed to open settings file.")
            )
        );

        let targets = vec![
            gtk::TargetEntry::new("text/uri-list", gtk::TargetFlags::OTHER_APP, 0)
        ];
        
        let app_runtime_clone = app_runtime.clone();
        ui.overlay.drag_dest_set(gtk::DestDefaults::ALL, &targets, gdk::DragAction::COPY);
        ui.overlay.connect_drag_data_received(move |_, _, _, _, selection, _, _| {
            // Since we only accept `text/uri-list`s here, we don't need to check first, we can simply
            // iterate through all of the accepted URIs.
            for file in selection.uris() {
                let file = gio::File::for_uri(&file);
                let file_name = if file.is_native() {
                    file.path().unwrap().display().to_string()
                } else {
                    file.uri().into()
                };
                
                app_runtime_clone.update_state_with(move |state| {
                    state.open_file_from_path(PathBuf::from(&file_name), None);
                });
            }
        });
        
        let db = Arc::new(RwLock::new(read_database()));
        
        AppOp {
            app_runtime,
            ui,
            settings,
            db,
            epub_book: None,
            open_page_sender: None,
            open_epub_book_sender: None,
        }
    }
    
    pub fn init(&mut self) {
        debug!("appop::init");
    
        // Save window state data to file when deleting the main window
        // (closing app).
        self.ui.main_window.connect_delete_event(glib::clone!(@strong self.app_runtime as app_runtime => move |window, _| {
            let window = window.clone();
            app_runtime.update_state_with(move |state| {
                let window = window.upcast_ref();
                let window_state = WindowState::from_window(window, state.ui.is_fullscreen);
                if let Err(err) = window_state.write() {
                    error!("Cannot save the window state: {:?}", err);
                }
            });

            Inhibit(false)
        }));
        
        self.open_page_sender = Some(self.open_page_message());
        self.open_epub_book_sender = Some(self.open_epub_book_message());
    }
    
    pub fn quit(&self) {
        debug!("appop::quit");
    }
    
    /// Open the next page.
    pub fn next_page(&mut self) {
        if let Some(book) = &mut self.epub_book {
            let num = book.doc.get_current_page() + 1;
            self.open_page_send(num);
        }
    }
    
    /// Open the previous page and check that the page number doesn't
    /// go below 0 because `usize` cannot handle that.
    pub fn previous_page(&mut self) {
        if let Some(book) = &mut self.epub_book {
            let current_page = book.doc.get_current_page();
            // Do nothing if current page is already at 0
            if current_page == 0 {
                return;
            }
            
            let num = current_page - 1;
            self.open_page_send(num);
        }
    }
    
    pub fn toggle_page_dialog(&mut self) {
        if let Some(book) = &mut self.epub_book {
            self.ui.page_dialog(
                self.app_runtime.clone(),
                book.doc.get_current_page(),
                book.doc.get_num_pages()
            );
        }
    }
    
    /// Open a new file from `PathBuf` in a new thread.
    /// Does nothing if the exact file is already open.
    pub fn open_file_from_path(&mut self, file: PathBuf, initial_page: Option<usize>) {
        // If the file is already open then do nothing
        if let Some(book) = self.epub_book.as_ref() {
            if book.file == file {
                return;
            }
        }
    
        debug!("Open file from path: {:?} | initial page: {:?}", file, initial_page);
        
        if let Some(extension) = get_extension_from_path(&file) {
            if extension == "epub" {
                // Open epub file in a new thread and then send
                // the result back to the main thread via a message.
                // Correct file extension so hide the left revealer if visible
                // and then show the loading indicator spinner.
                self.ui.left_revealer.set_reveal_child(false);
                self.ui.toggle_spinner(true);
                
                let tx = self.open_epub_book_sender.as_ref().unwrap().clone();
                thread::spawn(move || {
                    open_epub_file(file, initial_page, tx);
                });
            }
        }
    }
    
    /// Things to do after a file is open.
    /// Updates various UI elements.
    ///
    /// Opens the correct page from either the database or `book.initial_page`.
    pub fn handle_open_file(&mut self) {
        if let Some(book) = self.epub_book.as_mut() {
            // Open the page that was open previously if found in the db
            let db_row = self.db.read().get_by_row_file(&book.file);
            if let Some(row) = db_row {
                // If the `initial_page` was set in `EpubBook` then open that page
                // instead of looking for the page number in the database.

                debug!("Handle open file from path: {:?} | initial page: {:?}", book.file, book.initial_page);
                
                if let Some(num) = book.initial_page {
                    // Set the requested page or fall back to 0 in case of bad page number.
                    match book.doc.set_current_page(num) {
                        Ok(_) => {},
                        Err(_) => {
                            book.doc.set_current_page(0).expect("Cannot set the page to 0 o.O");
                        }
                    }
                } else {
                    let page_pref = self.settings.read().file.file_open_preference.to_usize();
                    match book.doc.set_current_page(row.current_page + page_pref) {
                        Ok(_) => {},
                        Err(_) => {
                            // Open the last page since the previous try failed.
                            book.doc.set_current_page(book.doc.get_num_pages() - 1).expect("Bad page o.O");
                        }
                    }
                }
            }
            
            self.ui.update_toc(self.app_runtime.clone(), book);
            self.ui.update(book, &self.settings.read());
            self.ui.scroller.vadjustment().set_value(0.0);
        }
    }
    
    /// Used for receiving the `EpubBook` result from another thread
    /// in which the file was opened.
    pub fn post_open_epub_book(&mut self, book: EpubBook) {
        self.epub_book = Some(book);
        self.handle_open_file();
        
        // Turn the spinner off
        self.ui.toggle_spinner(false);
    
        if !self.ui.left_revealer.is_visible() {
            self.ui.left_revealer.set_visible(true);
        }
    }
    
    /// Open Epub book page number specified with `num`.
    /// Does nothing if the requested page number is not valid or is already open.
    /// Moves the scrollbar to the top.
    ///
    /// TODO: Do most of this stuff in another fn and it should probably trigger at page end and file open in addition of opening a new page.
    pub fn open_page(&mut self, num: usize) {
        if let Some(book) = self.epub_book.as_mut() {
            // Do nothing if the requested page is already open
            if book.doc.get_current_page() != num {
                // Do nothing if trying to set a bad page number
                match book.doc.set_current_page(num) {
                    Ok(_) => {},
                    Err(_) => {
                        return;
                    }
                }
                
                // Update reader ui
                self.ui.update(book, &self.settings.read());
                self.ui.scroll_to_top();
    
                let file = book.file.clone();
                let filename = book.file.file_name().unwrap().to_os_string();
                // Update database
                self.update_db(file, filename, num);
            }
        }
    }
    
    pub fn update_db(&mut self, file: PathBuf, filename: OsString, current_page: usize) {
        let db = {
            self.db.read().clone()
        };
        
        // Add or replace `DatabaseRow` in `Database`.
        if let Some(mut rows) = db.rows {
            // New row that will be either replace an existing one
            // or be added as a new one
            let new_row = DatabaseRow {
                file,
                filename: filename.clone(),
                current_page
            };
        
            // Update or add a row in db
            if let Some(index) = rows.iter().position(|n| n.filename == filename) {
                // Replace some data in memory
                let _ = mem::replace(&mut rows[index], new_row);
                // Update the rows in db
                self.db.write().rows = Some(rows.clone());
            } else {
                // Add a new row into the db
                rows.push(new_row);
                // Update the rows in db
                self.db.write().rows = Some(rows.clone());
            }
        
            // Save to a file
            self.save_to_file();
            
            self.ui.update_recent(self.app_runtime.clone(), &rows);
        }
    }
    
    /// Saves the db into the file in a new thread
    pub fn save_to_file(&mut self) {
        debug!("appop::save_to_file");
        
        let db = self.db.clone();
        thread::spawn(move || {
            debug!("Saving to file in a new thread!");
            db.write().write_database();
        });
    }
}

/// Get a file extension from a filename
/// E.g: `foo.txt` -> `txt`.
fn get_extension_from_path(filename: &Path) -> Option<&str> {
    filename
        .extension()
        .and_then(OsStr::to_str)
}

/// This method is used in non-main thread.
/// It will open an epub file from `PathBuf` and sends the result
/// to the main thread.
fn open_epub_file(file: PathBuf, initial_page: Option<usize>, tx: glib::Sender<EpubBook>) {
    let epub_doc = EpubDoc::new(&file);
    match epub_doc {
        Ok(doc) => {
            let book = EpubBook {
                file,
                doc,
                initial_page
            };
    
            match tx.send(book) {
                Ok(_) => {},
                Err(e) => {
                    error!("Could not send `open_epub_book_message` from another thread!");
                    error!("{}", e);
                }
            }
        },
        Err(e) => {
            error!("Cannot open epub file: {:?}. Error: {}", file, e);
        }
    }
}