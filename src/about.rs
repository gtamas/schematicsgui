use std::path::PathBuf;

use chrono::prelude::*;
use relm4::gtk;
use relm4::gtk::gdk;
use relm4::gtk::gio;
use relm4::gtk::prelude::{GtkWindowExt, WidgetExt};

pub struct AppAboutDialog {}

impl AppAboutDialog {
    pub fn show() {
        // TODO: Add logo

        let dialog = gtk::AboutDialog::new();
        let now = chrono::offset::Local::now();

        const VERSION: &str = env!("CARGO_PKG_VERSION");
        const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
        const WEB: &str = env!("CARGO_PKG_HOMEPAGE");

        let mut authors: Vec<&str> = vec![];
        let author_iter = AUTHORS.split(",").into_iter();

        for author in author_iter {
            authors.push(author);
        }

        let file = gio::File::for_path(PathBuf::from("./resources/about.svg"));

        let img = match gdk::Texture::from_file(&file) {
            Ok(t) => t,
            Err(err) => panic!("Could not load logo file! {}", err),
        };

        dialog.set_logo(Some(&img));
        dialog.set_authors(&authors);
        dialog.set_copyright(Some(&format!(
            "Copyright {} by {}",
            now.year().to_string(),
            AUTHORS
        )));
        dialog.set_version(Some(VERSION));
        dialog.set_license_type(gtk::License::MitX11);
        dialog.set_website(Some(WEB));
        dialog.set_website_label("www.schematicsui.com");
        dialog.set_title(Some("About"));
        dialog.set_program_name(Some("Schematics GUI"));
        dialog.set_comments(Some("A GUI generator for Schematics projects"));

        dialog.show();
    }
}
