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

pub struct ThermalDataPos {
    pub x: usize,
    pub y: usize,
}

impl Default for ThermalDataPos {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
        }
    }
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

    pub fn get_min_max_pos(&self) -> (ThermalDataPos, ThermalDataPos) {
        let mut min_pos = ThermalDataPos::default();
        let mut max_pos = ThermalDataPos::default();
        let mut min_temp = f32::MAX;
        let mut max_temp = f32::MIN;
        for (i, pixel) in self.data.iter().enumerate() {
            let x = i % self.width;
            let y = i / self.width;
            let temp = *pixel;
            if temp < min_temp {
                min_temp = temp;
                min_pos.x = x;
                min_pos.y = y;
            }
            if temp > max_temp {
                max_temp = temp;
                max_pos.x = x;
                max_pos.y = y;
            }
        }
        (min_pos, max_pos)
    }
}
