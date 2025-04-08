pub mod colors;
pub mod color_clusterer;
pub mod weights;

use std::env;
use image::RgbImage;
use crate::color_clusterer::pallete;
use std::fs;
use std::path::Path;
use colors::{LAB, RGB};
use std::time::Instant;
use image::GenericImageView;


pub type Pixels = Vec<LAB>;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect(); // Skip program name

    if args.is_empty() {
        println!("No arguments provided.");
        return;
    }

    let mut only_colors = false;
    let mut hex = false;
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-c" | "--colors" => {
                only_colors = true
            },
            "-hx" | "--hex" => {
                hex = true
            },
            _ => {}
        }
    }

    
    let input_path = args[0].clone(); // Get image path passed by user
    let img_path = Path::new(&input_path);
    let output_dir = args[1].clone(); // Get output dir passed by user
    let output_dir_path = Path::new(&output_dir);


    // Create output directory if it doesn't exist
    if !output_dir_path.exists() && !only_colors {
        fs::create_dir(&output_dir).expect("Failed to create output directory");
    }


    if img_path.is_file() && is_image(&img_path) {
        if !only_colors {
            println!("Processing: {:?}", img_path);
        }

        // Generate palette
        pallete_from(&img_path, &output_dir_path, only_colors, hex);

    }       

}

// Check if a file is an image
fn is_image(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        matches!(ext.to_str().unwrap().to_lowercase().as_str(), "png" | "jpg" | "jpeg" | "bmp")
    } else {
        false
    }
}

// Generate a color palette
fn generate_palette(path: &Path) -> Result<(Vec<(LAB, f32)>, u128), Box<dyn std::error::Error>> {
    let mut img;
    img = image::open(path).expect("Failed to open image");
    img = img.resize(512,512, image::imageops::FilterType::CatmullRom);

    // Start timer
    let now = Instant::now();

    let pixels: Pixels = img
        .pixels()
        .map(|(_,_, pix)| LAB::from_rgb(pix[0], pix[1], pix[2]))
        .collect();

    let weightfn = weights::resolve_mood(&weights::Mood::Dominant);
    let mut output = pallete(&pixels, weightfn, 5 as u8);

    // Sort the output colors based on dominance
    output.sort_by(|(_, a), (_, b)| b.partial_cmp(a).unwrap());

    Ok((output, now.elapsed().as_millis()))
}


// Generate pallete files
fn pallete_from(path: &Path, output_dir: &Path, only_colors: bool, hex: bool) -> () {
    match generate_palette(path) {
        Ok((output, _)) => {
            for (color, _) in output.iter() {
                let rgb = RGB::from(color);
                let mut print_color = format!("{}",&rgb); 
                if hex { print_color = rgb.hex() }

                println!("{}", print_color);

            }
            if !only_colors {
                // Generate and save the palette image
                let output_img_path = Path::new(output_dir).join(
                    path.file_stem().unwrap().to_string_lossy().to_string() + "_palette.png",
                );
                create_pallete_file(&output_img_path, &output)
            }

        },
        Err(e) => print!("Error: {} WHILE trying to generate pallete from {}", e, path.display()),
    }
}


// Creates pallete file
fn create_pallete_file(output_img_path: &Path, colors: &Vec<(LAB, f32)>) {

    if colors.is_empty() {
        println!("No valid colors found in palette file.");
        return;
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

    img.save(output_img_path).expect("Failed to save palette image");
}
