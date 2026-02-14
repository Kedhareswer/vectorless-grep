// Run this with: cargo run --example generate_samples
// Or manually create images using any image editor

use image::{ImageBuffer, Rgb, RgbImage};

fn main() {
    // Create a simple 200x150 gradient image
    let mut img: RgbImage = ImageBuffer::new(200, 150);
    
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let r = (x as f32 / 200.0 * 255.0) as u8;
        let g = (y as f32 / 150.0 * 255.0) as u8;
        let b = 128;
        *pixel = Rgb([r, g, b]);
    }
    
    img.save("src-tauri/tests/fixtures/images/sample.png")
        .expect("Failed to save PNG");
    
    // Create a smaller JPEG
    let mut img_jpg: RgbImage = ImageBuffer::new(100, 100);
    for (x, y, pixel) in img_jpg.enumerate_pixels_mut() {
        let val = ((x + y) % 255) as u8;
        *pixel = Rgb([val, 100, 200 - val]);
    }
    
    img_jpg.save("src-tauri/tests/fixtures/images/sample.jpg")
        .expect("Failed to save JPG");
    
    println!("Sample images generated successfully!");
}
