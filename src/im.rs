use nih_plug::nih_dbg;
use nih_plug_egui::egui::ColorImage;

const BG_IMAGE: &[u8] = include_bytes!("../Resources/background.png");
const REEL_R_IMAGE: &[u8] = include_bytes!("../Resources/reel_r.png");
const REEL_L_IMAGE: &[u8] = include_bytes!("../Resources/reel_l.png");

#[derive(Clone)]
pub struct UiImages {
    pub background: ColorImage,
    pub reel_l: ColorImage,
    pub reel_r: ColorImage,
}

impl Default for UiImages {
    fn default() -> Self {
        Self {
            background: load_image_from_memory(BG_IMAGE).unwrap(),
            reel_r: load_image_from_memory(REEL_R_IMAGE).unwrap(),
            reel_l: load_image_from_memory(REEL_L_IMAGE).unwrap(),
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

pub fn load_image_from_memory(img: &[u8]) -> Result<ColorImage, image::ImageError> {
    let image = image::load_from_memory(img).expect("couldn't load");
    let size = [image.width() as _, image.height() as _];
    let image_buffer = image.to_rgba8();
    let pixels = image_buffer.as_flat_samples();
    Ok(ColorImage::from_rgba_unmultiplied(size, pixels.as_slice()))
}

fn get_path(path: &str) -> &std::path::Path {
    return std::path::Path::new(path);
}
