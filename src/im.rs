use std::io::Read;

use egui_extras::image::RetainedImage;
use nih_plug::nih_dbg;
use nih_plug_egui::egui::ColorImage;

#[derive(Clone)]
pub struct UiImages {
    pub background: ColorImage,
    pub reel: ColorImage,
}

impl Default for UiImages {
    fn default() -> Self {
        Self {
            //ARE PATHS RELATIVE TO THE BINARY AND NOT THE SOURCE??
            background: load_image_from_path("../../resources/background.png").unwrap(),
            reel: load_image_from_path("../../resources/reel.png").unwrap(),
        }
    }
}

pub fn load_image_from_path(path: &str) -> Result<ColorImage, image::ImageError> {
    nih_dbg!("start loading img");
    let image = image::io::Reader::open(get_path(path))?.decode()?;
    nih_dbg!("at least the path is good");
    let size = [image.width() as _, image.height() as _];
    let image_buffer = image.to_rgba8();
    let pixels = image_buffer.as_flat_samples();
    Ok(ColorImage::from_rgba_unmultiplied(size, pixels.as_slice()))
}

fn get_path(path: &str) -> &std::path::Path {
    return std::path::Path::new(path);
}
