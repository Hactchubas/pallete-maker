use image::{GenericImageView, RgbImage};
use std::{path::PathBuf, time::Instant};

use crate::{
    cli::ArgumentError,
    color_clusterer::pallete,
    colors::{LAB, RGB},
    weights,
};

pub type Pixels = Vec<LAB>;

pub struct Pallete {
    pub colors: Vec<(LAB, f32)>,
}

// Generate a color palette
pub fn generate_palette(
    img_path: &PathBuf,
    num_colors: &usize,
) -> Result<(Pallete, u128), Box<dyn std::error::Error>> {
    let mut img;
    img = image::open(img_path).expect("Failed to open image");
    img = img.resize(512, 512, image::imageops::FilterType::CatmullRom);

    // Start timer
    let now = Instant::now();

    let pixels: Pixels = img
        .pixels()
        .map(|(_, _, pix)| LAB::from_rgb(pix[0], pix[1], pix[2]))
        .collect();

    let weightfn = weights::resolve_mood(&weights::Mood::Dominant);
    let mut output = pallete(&pixels, weightfn, num_colors);

    // Sort the output colors based on dominance
    output.sort_by(|(_, a), (_, b)| b.partial_cmp(a).unwrap());

    Ok((Pallete { colors: output }, now.elapsed().as_millis()))
}

pub fn create_img(colors: &Vec<(LAB, f32)>) -> Result<RgbImage, ArgumentError> {
    let mut errors: Vec<String> = Vec::new();
    if colors.is_empty() {
        errors.push("No valid colors found in pallete".to_string());
        return Err(ArgumentError { errors });
    }

    let width = 500; // Image width
    let height = 100; // Height per color strip
    let img_height = colors.len() as u32 * height;

    let mut img = RgbImage::new(width, img_height);

    for (i, color) in colors.iter().enumerate() {
        let (color_lab, _) = color;
        let color_rgb = RGB::from(color_lab);
        let pixel_rgb = RGB::to_pixel(&color_rgb);

        let y_start = (i as u32) * height;
        for y in y_start..(y_start + height) {
            for x in 0..width {
                img.put_pixel(x, y, pixel_rgb);
            }
        }
    }

    Ok(img)
}
