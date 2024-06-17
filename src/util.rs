use crate::types::image_rotation::ImageRotation;
use eframe::epaint::{Color32, ColorImage};
use image::{GenericImage, Pixel, Rgb, RgbImage, Rgba};
use imageproc::rect::Rect;

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

pub fn image_to_egui_color_image(img: image::DynamicImage) -> ColorImage {
    let size = [img.width() as _, img.height() as _];
    let image_buffer = img.to_rgba8();
    let pixels = image_buffer.as_flat_samples();
    ColorImage::from_rgba_unmultiplied(size, pixels.as_slice())
}

pub fn draw_blended_rect_mut<I>(image: &mut I, rect: Rect, color: I::Pixel)
where
    I: GenericImage,
    I::Pixel: 'static,
{
    let image_bounds = Rect::at(0, 0).of_size(image.width(), image.height());
    if let Some(intersection) = image_bounds.intersect(rect) {
        for dy in 0..intersection.height() {
            for dx in 0..intersection.width() {
                let x = intersection.left() as u32 + dx;
                let y = intersection.top() as u32 + dy;
                let mut pixel = image.get_pixel(x, y); // added
                pixel.blend(&color); // added
                unsafe {
                    image.unsafe_put_pixel(x, y, pixel); // changed
                }
            }
        }
    }
}

pub fn draw_rounded_rect_mut<I>(image: &mut I, rect: Rect, radius: i32, color: I::Pixel)
where
    I: GenericImage,
    I::Pixel: 'static,
{
    fn is_point_in_rounded_rect(x: i32, y: i32, rect: Rect, radius: i32) -> bool {
        let x0 = rect.left() + radius;
        let x1 = rect.right() - radius;
        let y0 = rect.top() + radius;
        let y1 = rect.bottom() - radius;

        if x >= x0 && x <= x1 && y >= rect.top() && y <= rect.bottom() {
            return true;
        }
        if y >= y0 && y <= y1 && x >= rect.left() && x <= rect.right() {
            return true;
        }

        let corner =
            |x: i32, y: i32, cx: i32, cy: i32| (x - cx).pow(2) + (y - cy).pow(2) <= radius.pow(2);

        if corner(x, y, x0, y0)
            || corner(x, y, x1, y0)
            || corner(x, y, x0, y1)
            || corner(x, y, x1, y1)
        {
            return true;
        }

        false
    }
    let image_bounds = Rect::at(0, 0).of_size(image.width(), image.height());
    if let Some(intersection) = image_bounds.intersect(rect) {
        for dy in 0..intersection.height() {
            for dx in 0..intersection.width() {
                let x = intersection.left() + dx as i32;
                let y = intersection.top() + dy as i32;
                if is_point_in_rounded_rect(x, y, rect, radius) {
                    unsafe {
                        image.unsafe_put_pixel(x as u32, y as u32, color);
                    }
                }
            }
        }
    }
}

/// Overlays an old style film frame on top of the image
/// Added to video thumbnails to differentiate them from images
pub fn overlay_film_frame(img: image::DynamicImage) -> RgbImage {
    let mut img = img.to_rgba8();
    let height = img.height();
    let width = img.width();

    let black_semitransparent = Rgba([0, 0, 0, 128]);
    let white = Rgba([255, 255, 255, 255]);

    const FILM_STIPES_WIDTH: u32 = 32;

    draw_blended_rect_mut(
        &mut img,
        Rect::at(0, 0).of_size(FILM_STIPES_WIDTH, height),
        black_semitransparent,
    );
    draw_blended_rect_mut(
        &mut img,
        Rect::at((width - FILM_STIPES_WIDTH) as i32, 0).of_size(FILM_STIPES_WIDTH, height),
        black_semitransparent,
    );

    for i in (4..height + 32).step_by(30) {
        draw_rounded_rect_mut(
            &mut img,
            Rect::at(6, i as i32).of_size(FILM_STIPES_WIDTH - 12, 14),
            3,
            white,
        );

        draw_rounded_rect_mut(
            &mut img,
            Rect::at((width - FILM_STIPES_WIDTH) as i32 + 6, i as i32)
                .of_size(FILM_STIPES_WIDTH - 12, 14),
            3,
            white,
        );
    }
    image::DynamicImage::ImageRgba8(img).to_rgb8()
}
