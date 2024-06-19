use env_logger::Env;
use iced::widget::{button, column, text, Button, Column};
use iced::{executor, Application, Command, Element, Settings, Theme};
use log::info;

mod config;
mod db;
mod logging;
mod models;
mod utils;

/// Main function to start the SourceWatch application
/// It initializes the logger and starts the GUI
/// The GUI is an Iced application that allows the user to connect to different databases
fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    info!("Starting the SourceWatch application...");
    // db::mongo::connection::get_mongo_document();

    // gui::components::index::SourceWatchApp::run(Settings::default()).expect("Failed to start GUI");
}
