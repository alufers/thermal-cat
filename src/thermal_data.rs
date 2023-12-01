use eframe::epaint::{Color32, ColorImage};


#[derive(Clone)]
pub struct ThermalData {
    // Width in pixels
    pub width: usize,

    // Height in pixels
    pub height: usize,

    // Temperature data in degrees Kelvin
    pub data: Vec<f32>,
}

impl ThermalData {
    pub fn new(width: usize, height: usize, data: Vec<f32>) -> Self {
        Self {
            width,
            height,
            data,
        }
    }

    #[inline(always)]
    pub fn temperature_at(&self, x: usize, y: usize) -> f32 {
        self.data[y * self.width + x]
    }

    #[inline(always)]
    pub fn map_to_image<F: Fn(f32) -> Color32>(&self, callback: F) -> ColorImage {
        let mut img = ColorImage::new([self.width, self.height], Color32::BLACK);
        for (i, pixel) in img.pixels.iter_mut().enumerate() {
            let x = i % self.width;
            let y = i / self.width;
            *pixel = callback(self.temperature_at(x, y));
        }

        img
        
    }
}
