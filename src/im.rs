use std::io::Read;

use egui_extras::image::RetainedImage;
use nih_plug::nih_dbg;
use nih_plug_egui::egui::ColorImage;

#[derive(Clone)]
pub struct UiImages {
    pub background: ColorImage,
    pub reel_l: ColorImage,
    pub reel_r: ColorImage,
}

impl Default for UiImages {
    fn default() -> Self {
        Self {
            //ARE PATHS RELATIVE TO THE BINARY AND NOT THE SOURCE??
            background: load_image_from_path("/Users/Miredly/Resources/background.png").unwrap(),
            reel_r: load_image_from_path("/Users/Miredly/Resources/reel_r.png").unwrap(),
            reel_l: load_image_from_path("/Users/Miredly/Resources/reel_l.png").unwrap(),
        }
    }
}

pub fn load_image_from_path(path: &str) -> Result<ColorImage, image::ImageError> {
    nih_dbg!(path);
    nih_dbg!(std::env::current_exe().unwrap());
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
