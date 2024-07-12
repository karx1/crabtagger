use gtk::gdk::Screen;
use gtk::prelude::*;
use gtk::{glib, Application, ApplicationWindow, Builder, CssProvider, FileChooserButton};

const APP_ID: &str = "xyz.karx.CRABTAGGER";

fn main() -> glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();

    app.connect_startup(|_| load_css());
    app.connect_activate(build_ui);

    app.run()
}

fn load_css() {
    let provider = CssProvider::new();
    provider
        .load_from_data(include_bytes!("style.css"))
        .expect("Could not parse CSS data");

    gtk::StyleContext::add_provider_for_screen(
        &Screen::default().expect("Could not connect to display"),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn build_ui(app: &Application) {
    let builder = Builder::from_string(include_str!("crabtagger.glade"));

    let window: ApplicationWindow = builder.object("window").unwrap();
    let song_upload_button: FileChooserButton = builder.object("song_picker").unwrap();
    song_upload_button.connect_file_set(|b: &FileChooserButton| {
        println!("File picked!");
        println!("Pathname: {}", b.filename().unwrap().to_string_lossy());
    });

    // let window = ApplicationWindow::builder()
    // .application(app)
    // .title("Crabtagger")
    // .child(&label)
    // .build();

    window.set_application(Some(app));
    window.present();
}
