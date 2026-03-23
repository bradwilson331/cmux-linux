use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow};

mod ghostty;

const APP_ID: &str = "io.cmux.App";

fn main() {
    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("cmux")
        .default_width(800)
        .default_height(600)
        .build();

    window.present();
}
