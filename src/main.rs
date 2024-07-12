use gtk::prelude::*;
use gtk::{glib, Application, ApplicationWindow, Label};

const APP_ID: &str = "xyz.karx.CRABTAGGER";

fn main() -> glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(build_ui);

    app.run()
}

fn build_ui(app: &Application) {
    let label = Label::new(Some("Hello, world!"));

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Crabtagger")
        .child(&label)
        .build();

    window.present();
}
