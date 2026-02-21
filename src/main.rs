use image::io::Reader as ImageReader;
use image::{Luma, DynamicImage};
use imageproc::filter::laplacian;
use std::env;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: sharp-score <path_to_image>");
        std::process::exit(1);
    }

    let img_path = &args[1];
    if !Path::new(img_path).exists() {
        eprintln!("Error: File not found: {}", img_path);
        std::process::exit(1);
    }

    // 1. Load and convert to grayscale
    let img = match ImageReader::open(img_path).unwrap().decode() {
        Ok(img) => img,
        Err(e) => {
            eprintln!("Error decoding image: {}", e);
            std::process::exit(1);
        }
    };
    
    let gray_img = img.to_luma8();
    
    // 2. Apply Laplacian filter (detects edges/sharpness)
    // The Laplacian operator is great for edge detection.
    let laplacian_img = laplacian(&gray_img);
    
    // 3. Calculate Variance of the Laplacian response
    // Sum the pixels to get the mean
    let sum: f64 = laplacian_img.pixels().map(|p| p[0] as f64).sum();
    let count = laplacian_img.len() as f64;
    let mean = sum / count;
    
    // Sum of squared differences from the mean (variance)
    let variance: f64 = laplacian_img.pixels()
        .map(|p| {
            let diff = p[0] as f64 - mean;
            diff * diff
        })
        .sum::<f64>() / count;

    // 4. Normalize to 0-10 scale
    // Typical "sharp" images have Laplacian variance between 500 and 1500.
    // We use a logarithmic approach to make the scale feel natural:
    // Score = 10 * (log10(variance) / log10(MAX_EXPECTED_VARIANCE))
    // Let's use a simpler mapping for reliability:
    // sqrt(variance) is often used as a direct measure. 
    // In our experience, sqrt(variance) around 35 is very sharp.
    let intensity = variance.sqrt();
    let mut score = (intensity / 3.5).min(10.0);
    
    // If variance is extremely low, it's basically solid color or very blurry.
    if variance < 1.0 {
        score = 0.0;
    }

    println!("{:.1}", score);
}
