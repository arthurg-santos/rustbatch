mod app;
mod bridge;
mod state;
mod views;

use gtk4 as gtk;
use gtk::prelude::*;

fn main() -> glib::ExitCode {
    let app = gtk::Application::builder()
        .application_id("org.rustbatch.RustBatch")
        .build();

    app.connect_activate(app::build_ui);
    app.run()
}