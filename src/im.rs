use std::io::Read;

use egui_extras::image::RetainedImage;
use nih_plug::nih_dbg;
use nih_plug_egui::egui::ColorImage;

#[derive(Clone)]
pub struct UiImages {
    pub background: ColorImage,
}

impl Default for UiImages {
    fn default() -> Self {
        Self {
            //ARE PATHS RELATIVE TO THE BINARY AND NOT THE SOURCE??
            background: load_image_from_path("../../resources/background.png").unwrap(),
            // background: RetainedImage::from_image_bytes(
            //     "../resources/background.png",
            //     include_bytes!("../resources/background.png"),
            // )
            // .unwrap(),
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

// pub fn load_retained_image(path: &str) -> RetainedImage {
//     let mut buffer = vec![];
//     std::fs::File::open(path)
//         .unwrap()
//         .read_to_end(&mut buffer)
//         .unwrap();
//     return RetainedImage::from_image_bytes(
//         path.clone(),
//         include_bytes!(String::from_buffer(path)),
//     )
//     .unwrap();
// }

fn get_path(path: &str) -> &std::path::Path {
    return std::path::Path::new(path);
}
