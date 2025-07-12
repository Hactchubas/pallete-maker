use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    path::{Path, PathBuf},
};

pub struct Request {
    pub(crate) img_path: PathBuf,
    pub(crate) output_path: Option<PathBuf>,
    pub(crate) rgb: bool,
    pub(crate) hex: bool,
    pub(crate) silent: bool,
    pub(crate) time: bool,
    pub(crate) num_colors: usize,
}

#[derive(Debug)]
pub struct ArgumentError {
    pub errors: Vec<String>,
}

impl Display for ArgumentError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(f, "Argument validation error:")?;
        for (i, error) in self.errors.iter().enumerate() {
            writeln!(f, "\t{}. {}", i + 1, error)?;
        }
        Ok(())
    }
}

impl Error for ArgumentError {}

// Check if a file is an image
fn is_image(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        matches!(
            ext.to_str().unwrap().to_lowercase().as_str(),
            "png" | "jpg" | "jpeg" | "bmp" | "gif" | "webp"
        )
    } else {
        false
    }
}

pub fn parse_args(args: Vec<String>) -> Result<Request, ArgumentError> {
    let mut errors: Vec<String> = Vec::new();

    // Check input given not empty
    if args.is_empty() {
        errors.push("Missing required image path".to_string());
        return Err(ArgumentError { errors });
    }

    let img_path = PathBuf::from(&args[0]);
    let mut output_path: Option<PathBuf> = None;
    let mut rgb = false;
    let mut hex = false;
    let mut silent = false;
    let mut time = false;
    let mut num_colors = 5;

    // Validate image path
    if !img_path.exists() {
        errors.push(format!(
            "Image path '{}' does not exist",
            img_path.display()
        ));
    } else if !img_path.is_file() {
        errors.push(format!("Image path '{}' is not a file", img_path.display()));
    } else if !is_image(&img_path) {
        errors.push(format!(
            "File '{}' is not a supported image format",
            img_path.display()
        ));
    }

    // Process remaining arguments
    for i in 1..args.len() {
        match args[i].as_str() {
            "-rgb" => rgb = true,
            "-hx" => hex = true,
            "-silent" => silent = true,
            "-time" => time = true,
            "-num" => {
                if let Some(ncolors_str) = args.get(i + 1) {
                    match ncolors_str.parse::<usize>() {
                        Ok(num) => num_colors = num,
                        Err(_) => errors.push(format!(
                            "Invalid use of -num, it requires a valid number (1-12)"
                        )),
                    }
                } else {
                    errors.push(format!("-num flag requires a number argument"));
                }
            }
            _ => {
                // Skip if this argument was consumed by a previous flag
                if i > 1 && args[i - 1] == "-num" {
                    continue;
                }
                // If it's not a flag and we haven't set output_path yet, treat it as directory path
                if output_path.is_none() {
                    let path = PathBuf::from(&args[i]);
                    if !path.exists() {
                        errors.push(format!("Path '{}' does not exist", path.display()));
                    } else {
                        output_path = Some(path);
                    }
                } else {
                    errors.push(format!("Unknown argument: '{}'", args[i]));
                }
            }
        }
    }

    // Validate silent flag can only be used without color flags
    if silent && (rgb || hex) {
        errors.push("Silent flag cannot be used with RGB or hex flags".to_string());
    }

    // Apply flag logic: if no flags are given, default to rgb = true
    if !rgb && !hex && !silent && !time {
        rgb = true;
    }

    // Return errors if any were found
    if !errors.is_empty() {
        return Err(ArgumentError { errors });
    }

    Ok(Request {
        img_path,
        output_path,
        rgb,
        hex,
        silent,
        time,
        num_colors,
    })
}
