use gtk::gdk::Screen;
use gtk::gdk_pixbuf::Pixbuf;
use gtk::gio::{Cancellable, MemoryInputStream};
use gtk::glib::{self, clone, Bytes};
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Builder, CssProvider, Entry, FileChooserButton, Image};
use id3::{Tag, TagLike};

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
    let title_entry: Entry = builder.object("title_entry").unwrap();
    let artist_entry: Entry = builder.object("artist_entry").unwrap();
    let album_entry: Entry = builder.object("album_entry").unwrap();
    let cover: Image = builder.object("cover").unwrap();
    let image_upload_button: FileChooserButton = builder.object("image_upload").unwrap();

    song_upload_button.connect_file_set(
        clone!(@weak title_entry, @weak artist_entry, @weak album_entry, @weak cover => move |b: &FileChooserButton| {
            println!("File picked!");
            let filename = b.filename().unwrap();
            println!("Pathname: {}", filename.to_string_lossy());
            let tag = Tag::read_from_path(&filename).unwrap_or_default();
            for frame in tag.frames() {
                println!("{frame}");
            }

            if let Some(title) = tag.title() {
                title_entry.set_text(title);
            } else {
                title_entry.set_text("");
            }

            if let Some(artist) = tag.artist() {
                artist_entry.set_text(artist);
            } else {
                artist_entry.set_text("");
            }

            if let Some(album) = tag.album() {
                album_entry.set_text(album);
            } else {
                album_entry.set_text("");
            }

            // if let Some(picture) = tag.pictures().cloned().nth(0) {
                // println!("{}", picture.mime_type);
            // }
            let picture_maybe = tag.pictures().nth(0); // borrow checker moment
            if let Some(picture) = picture_maybe {
                println!("{}", picture.mime_type);
                let bytes = Bytes::from(&picture.data);
                let stream = MemoryInputStream::from_bytes(&bytes);
                let pixbuf = Pixbuf::from_stream_at_scale(&stream, 200, 200, true, Cancellable::NONE);
                cover.set_from_pixbuf(pixbuf.ok().as_ref());
            } else {
                cover.set_icon_name(Some("audio-card"));
            }
        }),
    );

    image_upload_button.connect_file_set(clone!(@weak cover => move |b: &FileChooserButton| {
        println!("Image picked!");
        let filename = b.filename().unwrap();
        let pixbuf = Pixbuf::from_file_at_scale(&filename, 200, 200, true);
        cover.set_from_pixbuf(pixbuf.ok().as_ref());
    }));

    window.set_application(Some(app));
    window.present();
}
