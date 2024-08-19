use anyhow::Error;
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
use multitag::data::Picture;
use multitag::data::Timestamp;
use multitag::Tag;

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
    let year_entry: Entry = builder.object("year").unwrap();
    let month_entry: Entry = builder.object("month").unwrap();
    let day_entry: Entry = builder.object("day").unwrap();

    song_upload_button.connect_file_set(
        clone!(@weak title_entry, @weak artist_entry, @weak album_entry, @weak cover, @weak window, @weak year_entry, @weak month_entry, @weak day_entry => move |b: &FileChooserButton| {
            let do_thing = || -> Result<(), Error> {
                println!("File picked!");
                let filename = b.filename().unwrap();
                println!("Pathname: {}", filename.to_string_lossy());
                let tag = Tag::read_from_path(&filename).unwrap();

                if let Some(title) = tag.title() {
                    title_entry.set_text(title);
                } else {
                    title_entry.set_text("");
                }

                if let Some(artist) = tag.artist() {
                    artist_entry.set_text(&artist);
                } else {
                    artist_entry.set_text("");
                }

                if let Some(album) = tag.get_album_info() {
                    album_entry.set_text(&album.title.unwrap_or_default());
                    let picture_maybe = album.cover;
                    if let Some(picture) = picture_maybe {
                        let bytes = Bytes::from(&picture.data);
                        let stream = MemoryInputStream::from_bytes(&bytes);
                        let pixbuf = Pixbuf::from_stream_at_scale(&stream, 200, 200, true, Cancellable::NONE)?;
                        cover.set_from_pixbuf(Some(&pixbuf));
                    } else {
                        cover.set_icon_name(Some("audio-card"));
                    }
                } else {
                    album_entry.set_text("");
                }

                if let Some(time) = tag.date() {
                    // todo: wait for remove methods in multitag
                    year_entry.set_text(&Some(time.year).filter(|n: &i32| *n != 0).map(|n| format!("{n:04}")).unwrap_or_default());
                    month_entry.set_text(&time.month.filter(|n: &u8| *n != 0).map(|n| format!("{n:02}")).unwrap_or_default());
                    day_entry.set_text(&time.day.filter(|n: &u8| *n != 0).map(|n| format!("{n:02}")).unwrap_or_default());
                } else {
                    year_entry.set_text("");
                    month_entry.set_text("");
                    day_entry.set_text("");
                }
                Ok(())
            };

            match do_thing() {
                Ok(()) => {}
                Err(e) => {
                    message(
                        format!("Encountered an error while saving: {e}"),
                        Some(&window),
                        MessageType::Error,
                        ButtonsType::Ok
                    );
                }
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
        clone!(@weak song_upload_button, @weak window, @weak title_entry, @weak artist_entry, @weak album_entry, @weak image_upload_button, @weak year_entry, @weak month_entry, @weak day_entry => move |_| {
            let do_thing = || -> Result<(), Error> {
                if let Some(filename) = song_upload_button.filename() {
                    let mut tag = Tag::read_from_path(&filename)?;
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
                        let mut album_info = tag.get_album_info().unwrap_or_default();
                        album_info.title = Some(album_entry.text().to_string());

                        tag.set_album_info(album_info)?;
                    }

                    if let Some(cover_file) = image_upload_button.file() {
                        if let Ok((bytes, _)) = cover_file.load_bytes(Cancellable::NONE) {
                            let bytes_owned = bytes.to_vec();
                            let mut album_info = tag.get_album_info().unwrap_or_default();
                            // all mime types should be sniffable so this is ok
                            let mime_type = bytes_owned.sniff_mime_type().unwrap_or("image/jpeg").to_string();
                            album_info.cover = Some(Picture {
                                mime_type,
                                data: bytes_owned
                            });
                            tag.set_album_info(album_info)?;
                        }
                    }

                    if let Ok(year) = year_entry.text().parse::<i32>() {
                        let time = Timestamp {
                            year,
                            month: month_entry.text().parse().ok(),
                            day: day_entry.text().parse().ok(),
                            ..Default::default()
                        };
                        tag.set_date(time);
                    } else if !month_entry.text().is_empty() || !day_entry.text().is_empty() {
                        message("Please enter a year!", Some(&window), MessageType::Error, ButtonsType::Ok);
                        return Ok(());
                    } else {
                        // meaning year, month, and day are all empty
                        tag.set_date(Timestamp::default());
                    }

                    tag.write_to_path(filename)?;
                } else {
                    message("No song selected!", Some(&window), MessageType::Error, ButtonsType::Ok);
                }
                Ok(())
            };

            match do_thing() {
                Ok(()) => {
                    song_upload_button.unselect_all();
                    image_upload_button.unselect_all();
                    title_entry.set_text("");
                    artist_entry.set_text("");
                    album_entry.set_text("");
                    cover.set_from_icon_name(Some("audio-card"), IconSize::Button);
                    year_entry.set_text("");
                    month_entry.set_text("");
                    day_entry.set_text("");
                    message("Saved successfully!", Some(&window), MessageType::Info, ButtonsType::Ok);
                },
                Err(e) => {
                    message(
                        format!("Encountered an error while saving: {e}"),
                        Some(&window),
                        MessageType::Error,
                        ButtonsType::Ok
                    );
                }
            }
        }),
    );

    year_entry.connect_insert_text(|entry, text, position| {
        entry_disallow(entry, text, |c: char| !c.is_ascii_digit(), position);
    });
    month_entry.connect_insert_text(|entry, text, position| {
        entry_disallow(entry, text, |c: char| !c.is_ascii_digit(), position);
    });
    day_entry.connect_insert_text(|entry, text, position| {
        entry_disallow(entry, text, |c: char| !c.is_ascii_digit(), position);
    });

    window.set_application(Some(app));
    window.present();
}

fn entry_disallow<F: FnMut(char) -> bool, S: AsRef<str>>(
    entry: &Entry,
    text: S,
    mut predicate: F,
    position: &mut i32,
) {
    if text.as_ref().contains(&mut predicate) {
        glib::signal::signal_stop_emission_by_name(entry, "insert-text");
        entry.insert_text(&text.as_ref().replace(predicate, ""), position);
    }
}
