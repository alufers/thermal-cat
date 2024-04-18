use crate::types::image_rotation::ImageRotation;
use eframe::epaint::{Color32, ColorImage};
use image::{Rgb, Rgba};

pub fn rotate_image(img: ColorImage, rotation: ImageRotation) -> ColorImage {
    if rotation == ImageRotation::None {
        return img;
    }
    let mut new_img = ColorImage::new(
        match rotation {
            ImageRotation::None => img.size,
            ImageRotation::Clockwise90 => [img.size[1], img.size[0]],
            ImageRotation::Clockwise180 => img.size,
            ImageRotation::Clockwise270 => [img.size[1], img.size[0]],
        },
        Color32::BLACK,
    );

    for (i, pixel) in img.pixels.iter().enumerate() {
        let x = i % img.size[0];
        let y = i / img.size[0];

        let new_x = match rotation {
            ImageRotation::None => x,
            ImageRotation::Clockwise90 => y,
            ImageRotation::Clockwise180 => img.size[0] - x - 1,
            ImageRotation::Clockwise270 => img.size[1] - y - 1,
        };
        let new_y = match rotation {
            ImageRotation::None => y,
            ImageRotation::Clockwise90 => img.size[0] - x - 1,
            ImageRotation::Clockwise180 => img.size[1] - y - 1,
            ImageRotation::Clockwise270 => x,
        };

        new_img.pixels[new_y * new_img.size[0] + new_x] = *pixel;
    }
    new_img
}

pub fn pathify_string(s: String) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| match c {
            'a'..='z' | '0'..='9' => c,
            _ => '_',
        })
        .collect()
}

pub fn rgba8_to_rgb8(
    input: image::ImageBuffer<Rgba<u8>, Vec<u8>>,
) -> image::ImageBuffer<Rgb<u8>, Vec<u8>> {
    let width = input.width() as usize;
    let height = input.height() as usize;

    // Get the raw image data as a vector
    let input: &Vec<u8> = input.as_raw();

    // Allocate a new buffer for the RGB image, 3 bytes per pixel
    let mut output_data = vec![0u8; width * height * 3];

    let mut i = 0;
    // Iterate through 4-byte chunks of the image data (RGBA bytes)
    for chunk in input.chunks(4) {
        // ... and copy each of them to output, leaving out the A byte
        output_data[i..i + 3].copy_from_slice(&chunk[0..3]);
        i += 3;
    }

    // Construct a new image
    image::ImageBuffer::from_raw(width as u32, height as u32, output_data).unwrap()
}
