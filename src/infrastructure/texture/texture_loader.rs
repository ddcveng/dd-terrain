use std::path::Path;

use glium::texture::{RawImage2d, Texture2d};

use crate::config;

pub fn texture_from_file(filename: &str, facade: &glium::Display) -> Texture2d {
    let file_path = Path::new(config::ASSETS_PATH).join(filename);
    let img = match image::open(file_path) {
        Ok(img) => img,
        Err(img_error) => panic!("failed to open file {filename} - {img_error}"),
    };

    // Pixels in the image buffer are ordered top-down and left to right
    // but glium texture requires the pixels to be ordered bottom-up and left to right
    // so we have to flip the texture vertically
    let flipped_img = img.flipv();
    let rgb_image_buffer = flipped_img.to_rgb32f();

    let dimensions = rgb_image_buffer.dimensions();
    let pixels_raw = rgb_image_buffer.into_raw();

    let texture_data_source = RawImage2d::from_raw_rgb(pixels_raw, dimensions);
    let texture = match Texture2d::new(facade, texture_data_source) {
        Ok(tex) => tex,
        Err(texture_creation_error) => {
            panic!("failed to create texture - {texture_creation_error}!")
        }
    };

    texture
}
