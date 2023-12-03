use eframe::epaint::{Color32, ColorImage};

use crate::temperature::{Temp, TempRange};

#[derive(Clone)]
pub struct ThermalData {
    // Width in pixels
    pub width: usize,

    // Height in pixels
    pub height: usize,

    // Temperature data in degrees Kelvin
    pub data: Vec<Temp>,
}

pub struct ThermalDataPos {
    pub x: usize,
    pub y: usize,
}

impl Default for ThermalDataPos {
    fn default() -> Self {
        Self { x: 0, y: 0 }
    }
}

impl ThermalData {
    pub fn new(width: usize, height: usize, data: Vec<Temp>) -> Self {
        Self {
            width,
            height,
            data,
        }
    }

    #[inline(always)]
    pub fn temperature_at(&self, x: usize, y: usize) -> Temp {
        self.data[y * self.width + x]
    }

    #[inline(always)]
    pub fn map_to_image<F: Fn(Temp) -> Color32>(&self, callback: F) -> ColorImage {
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
        let mut min_temp = Temp::MAX;
        let mut max_temp = Temp::MIN;
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

#[derive(Clone, Debug)]
pub struct ThermalDataHistogramPoint {
    pub temperature: Temp,
    pub factor: f32,
}

pub struct ThermalDataHistogram {
    pub points: Vec<ThermalDataHistogramPoint>,
}

impl ThermalDataHistogram {
    pub fn from_thermal_data(data: &ThermalData, range: TempRange, num_buckets: usize) -> Self {
        let mut buckets = vec![0; num_buckets];

        for temp in &data.data {
            let bucket = range.factor(*temp) * (num_buckets as f32);
            let bucket = bucket as usize;
            if bucket >= num_buckets {
                continue;
            }
            buckets[bucket] += 1;
        }

        let total_pixels = data.data.len();

        let mut points = Vec::new();
        for (i, bucket) in buckets.iter().enumerate() {
            let factor = *bucket as f32 / total_pixels as f32;
            let temperature =
                range.min + (range.max - range.min) * ((i as f32 + 0.5) / num_buckets as f32);
            points.push(ThermalDataHistogramPoint {
                temperature,
                factor,
            });
        }
        Self { points }
    }
}
