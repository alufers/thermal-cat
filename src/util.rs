use eframe::epaint::{Color32, ColorImage};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ImageRotation {
    None,
    Clockwise90,
    Clockwise180,
    Clockwise270,
}

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
    return new_img;
}
