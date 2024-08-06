use audiotags::types::{MimeType, Picture};
use audiotags::Tag;
use gio::prelude::FileExt;
use gtk::gdk::Screen;
use gtk::gdk_pixbuf::Pixbuf;
use gtk::gio::{self, Cancellable, MemoryInputStream};
use gtk::glib::{self, clone, Bytes};
use gtk::prelude::*;
use gtk::{
    Application, ApplicationWindow, Builder, Button, CssProvider, Entry, FileChooserButton,
    IconSize, Image, Window,
};
use gtk::{ButtonsType, DialogFlags, MessageDialog, MessageType};
use mime_sniffer::MimeTypeSniffer;

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

fn message<S: AsRef<str>, P: IsA<Window>>(
    text: S,
    parent: Option<&P>,
    msg_type: MessageType,
    button_type: ButtonsType,
) {
    // partially stolen from https://git.lemonsh.moe/lemon/zdiu
    let dialog = MessageDialog::new(
        parent,
        DialogFlags::MODAL,
        msg_type,
        button_type,
        text.as_ref(),
    );
    dialog.run();
    dialog.close();
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
    let save_button: Button = builder.object("save").unwrap();

    song_upload_button.connect_file_set(
        clone!(@weak title_entry, @weak artist_entry, @weak album_entry, @weak cover, @weak window => move |b: &FileChooserButton| {
            println!("File picked!");
            let filename = b.filename().unwrap();
            println!("Pathname: {}", filename.to_string_lossy());
            let tag = Tag::new().read_from_path(&filename).unwrap();

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
                album_entry.set_text(album.title);
            } else {
                album_entry.set_text("");
            }

            let picture_maybe = tag.album_cover();
            if let Some(picture) = picture_maybe {
                // if picture.mime_type == "image/webp" {
                    // message("WebP Image detected. Please replace with a JPEG/PNG image for maximum compatibility!", Some(&window), MessageType::Error, ButtonsType::Ok);
                // }
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

    save_button.connect_clicked(
        clone!(@weak song_upload_button, @weak window, @weak title_entry, @weak artist_entry, @weak album_entry, @weak image_upload_button => move |_| {
            if let Some(filename) = song_upload_button.filename() {
                let mut tag = Tag::new().read_from_path(&filename).unwrap();
                let title = title_entry.text();
                let artist = artist_entry.text();
                let album = album_entry.text();
                if !title.is_empty() {
                    tag.set_title(&title_entry.text());

                }
                if !artist.is_empty() {
                    tag.set_artist(&artist_entry.text());
                }
                if !album.is_empty() {
                    tag.set_album_title(&album_entry.text());
                }

                if let Some(cover_file) = image_upload_button.file() {
                    if let Ok((bytes, _)) = cover_file.load_bytes(Cancellable::NONE) {
                        let bytes_owned = bytes.to_vec();
                        let mime_type = MimeType::try_from(bytes_owned.sniff_mime_type().unwrap_or("image/jpeg")).unwrap_or(MimeType::Jpeg);
                        println!("{}", String::from(mime_type));
                        tag.set_album_cover(Picture {
                            mime_type,
                            data: &bytes_owned
                        });
                    }
                }

                if let Some(file_str) = filename.to_str() {
                    let res = tag.write_to_path(file_str);
                    if let Err(e) = res {
                        message(
                            format!("Encountered an error while saving: {}", e),
                            Some(&window),
                            MessageType::Error,
                            ButtonsType::Ok
                        );
                    } else {
                        song_upload_button.unselect_all();
                        image_upload_button.unselect_all();
                        title_entry.set_text("");
                        artist_entry.set_text("");
                        album_entry.set_text("");
                        cover.set_from_icon_name(Some("audio-card"), IconSize::Button);
                        message("Saved successfully!", Some(&window), MessageType::Info, ButtonsType::Ok);
                    }
                } else {
                    message("Filenames containing non-unicode characters are not supported, please rename the file and try again!", Some(&window), MessageType::Error, ButtonsType::Ok);
                }
            } else {
                message("No song selected!", Some(&window), MessageType::Error, ButtonsType::Ok);
            }
        }),
    );

    window.set_application(Some(app));
    window.present();
}
