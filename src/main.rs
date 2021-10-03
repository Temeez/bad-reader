#[macro_use]
extern crate cascade;
#[macro_use]
extern crate log;
extern crate select;

mod app;
mod ui;
mod appop;

use std::error::Error;
use std::path::Path;
use std::fs::File;
use clap::{App, Arg};
use gtk::prelude::{ApplicationExt, ApplicationExtManual};
use crate::app::utils::working_dir;

pub const PROGRAM_NAME: &str = "Bad Reader";
pub const VERSION: &str = "1.0";
pub const ABOUT: &str = "Bad epub reader made with Rust.";
pub const LOG_FILE: &str = "bad-reader.log";

fn setup_logging() -> Result<(), fern::InitError> {
    let path = Path::new(&working_dir(LOG_FILE)).to_path_buf();
    
    // Clear the log file if it exists
    if path.as_path().exists() {
        let f = File::create(&path).expect("Cannot open log file");
        f.set_len(0).expect("Cannot set file length to 0");
    }
    
    fern::Dispatch::new()
        // Perform allocation-free log formatting
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        // Add blanket level filter -
        .level(log::LevelFilter::Info)
        // - and per-module overrides
        .level_for("bad_reader", log::LevelFilter::Debug)
        // Output to stdout, files, and other Dispatch configurations
        .chain(std::io::stdout())
        .chain(fern::log_file(path)?)
        // Apply globally
        .apply()?;
    
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    setup_logging().expect("failed to initialize logging.");
    
    let matches = App::new(PROGRAM_NAME)
        .version(VERSION)
        .author("Teemu N.")
        .about(ABOUT)
        .arg(Arg::new("file")
                 .short('f')
                 .long("file")
                 .value_name("FILE")
                 .about("Set the epub file to open.")
                 .takes_value(true))
        .arg(Arg::new("page")
                 .short('p')
                 .long("page")
                 .value_name("PAGE")
                 .about("Optianlly open a specific page. Page numbers start from zero (0).")
                 .takes_value(true))
        .get_matches();
        
    let application = gtk::Application::new(
        Some("com.github.temeez.badreader"),
        gio::ApplicationFlags::FLAGS_NONE,
    );
    
    // Make the gtk app acknowledge the arguments defined above.
    // Otherwise it'll whine `Unknown option -f` and does nothing.
    application.add_main_option(
        "file",
        glib::Char::from(b'f'),
        glib::OptionFlags::IN_MAIN,
        glib::OptionArg::Filename,
        "dummy",
        Some("dummy")
    );
    application.add_main_option(
        "page",
        glib::Char::from(b'p'),
        glib::OptionFlags::IN_MAIN,
        glib::OptionArg::Int,
        "dummy",
        Some("dummy")
    );
    
    application.connect_startup(move |application| {
        app::on_startup(application, &matches);
    });
    
    application.run();
    
    Ok(())
}