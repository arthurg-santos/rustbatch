use gtk4 as gtk;
use gtk::prelude::*;
use std::path::PathBuf;
use rustbatch_core::model::ProcessConfig;
use rustbatch_core::pipeline::process_image;

#[derive(Clone)]
pub struct PreviewWidget {
    pub widget: gtk::Box,
    picture: gtk::Picture,
}

impl PreviewWidget {
    pub fn new() -> Self {
        let picture = gtk::Picture::new();
        picture.set_can_shrink(true);
        picture.set_size_request(400, 300);

        let label = gtk::Label::new(Some("Pré-visualização"));
        label.add_css_class("dim-label");

        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 4);
        vbox.set_margin_top(8);
        vbox.set_margin_bottom(8);
        vbox.set_margin_start(8);
        vbox.set_margin_end(8);
        
        vbox.append(&label);
        vbox.append(&picture);

        Self { widget: vbox, picture }
    }

    pub fn update(&self, source: Option<&PathBuf>, config: &ProcessConfig) {
        let Some(path) = source else {
            self.picture.set_paintable(gtk::gdk::Paintable::NONE);
            return;
        };

        match render_preview(path, config) {
            Ok(texture) => self.picture.set_paintable(Some(&texture)),
            Err(e) => {
                log::warn!("Falha na pré-visualização: {e}");
                self.picture.set_paintable(gtk::gdk::Paintable::NONE);
            }
        }
    }
}

fn render_preview(
    path: &PathBuf,
    config: &ProcessConfig,
) -> Result<gtk::gdk::Texture, Box<dyn std::error::Error>> {
    let img = process_image(path, config)?;
    let thumb = img.thumbnail(400, 300);
    let rgba = thumb.to_rgba8();
    let bytes = glib::Bytes::from_owned(rgba.clone().into_raw());
    let texture = gtk::gdk::MemoryTexture::new(
        rgba.width() as i32,
        rgba.height() as i32,
        gtk::gdk::MemoryFormat::R8g8b8a8,
        &bytes,
        (rgba.width() * 4) as usize,
    );
    Ok(texture.upcast())
}