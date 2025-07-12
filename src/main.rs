use std::error::Error;

use crate::{
    cli::parse_args,
    colors::RGB,
    pallete::{create_img, generate_palette},
};

pub mod cli;
pub mod color_clusterer;
pub mod colors;
pub mod pallete;
pub mod weights;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect(); // Skip program name

    let parsed_request = parse_args(args)?;
    let img_path = &parsed_request.img_path;
    let num_colors = &parsed_request.num_colors;

    let (pallete, generating_time) = generate_palette(img_path, num_colors)?;

    // Display colors based on flags
    if parsed_request.rgb || parsed_request.hex {
        for (i, (color_lab, weight)) in pallete.colors.iter().enumerate() {
            let color_rgb = RGB::from(color_lab);

            if parsed_request.rgb {
                println!(
                    "Color {}: RGB({}, {}, {}) - Weight: {:.2}%",
                    i + 1,
                    color_rgb.r,
                    color_rgb.g,
                    color_rgb.b,
                    weight * 100.0
                );
            }

            if parsed_request.hex {
                println!(
                    "Color {}: #{:02X}{:02X}{:02X} - Weight: {:.2}%",
                    i + 1,
                    color_rgb.r,
                    color_rgb.g,
                    color_rgb.b,
                    weight * 100.0
                );
            }
        }
    }

    // Only create image if output path is provided
    if let Some(output_path) = &parsed_request.output_path {
        let img = create_img(&pallete.colors)?;

        let final_output_path = if output_path.is_dir() {
            // If output is a directory, generate filename with palette_ prefix
            let img_filename = img_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("image");
            let img_extension = img_path
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("png");
            let palette_filename = format!("palette_{}.{}", img_filename, img_extension);
            output_path.join(palette_filename)
        } else {
            // If output is a file path, check if it already exists
            if output_path.exists() {
                return Err(
                    format!("Output file '{}' already exists", output_path.display()).into()
                );
            }
            output_path.clone()
        };

        img.save(&final_output_path)?;
        if !parsed_request.silent {
            println!("Palette image saved to: {}", final_output_path.display());
        }
    }

    if !parsed_request.silent && parsed_request.time {
        println!("Palette generated in {}ms", generating_time);
    }

    Ok(())
}
