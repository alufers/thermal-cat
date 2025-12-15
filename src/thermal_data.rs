use eframe::epaint::{Color32, ColorImage};

use crate::{
    temperature::{Temp, TempRange, TemperatureUnit},
    types::image_rotation::ImageRotation,
};

#[derive(Clone)]
pub struct ThermalData {
    // Width in pixels
    pub width: usize,

    // Height in pixels
    pub height: usize,

    // Temperature data in degrees Kelvin
    pub data: Vec<Temp>,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ThermalDataPos {
    pub x: usize,
    pub y: usize,
}

impl ThermalDataPos {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
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
        if x >= self.width {
            panic!("x out of bounds ({} >= {})", x, self.width);
        }
        if y >= self.height {
            panic!("y out of bounds ({} >= {})", y, self.height);
        }
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

    pub fn corrected(&self, ambient : f32, emissivity : f32) -> Self {
		
        let amb_pow4 = (1.0-emissivity)*ambient.powi(4);
        let (width, height) = (self.width, self.height);
        
        let mut data: Vec<Temp> = vec![Temp::new(0.0); width * height];
        for (i, pixel) in self.data.iter().enumerate() {
			data[i]=Temp::from_unit(TemperatureUnit::Kelvin, (((*pixel).to_unit(TemperatureUnit::Kelvin).powi(4)-amb_pow4)/ emissivity.max(1e-6)).powf(0.25));
		}

        Self {
            width,
            height,
            data,
        }
    }
    
    pub fn rotated(&self, rotation: ImageRotation) -> Self {
        if rotation == ImageRotation::None {
            return self.clone();
        }
        let (width, height) = match rotation {
            ImageRotation::None => (self.width, self.height),
            ImageRotation::Clockwise90 => (self.height, self.width),
            ImageRotation::Clockwise180 => (self.width, self.height),
            ImageRotation::Clockwise270 => (self.height, self.width),
        };

        let mut data: Vec<Temp> = vec![Temp::new(0.0); width * height];
        for (i, pixel) in self.data.iter().enumerate() {
            let x = i % self.width;
            let y = i / self.width;
            let (x, y) = match rotation {
                ImageRotation::None => (x, y),
                ImageRotation::Clockwise90 => (y, self.width - x - 1),
                ImageRotation::Clockwise180 => (self.width - x - 1, self.height - y - 1),
                ImageRotation::Clockwise270 => (self.height - y - 1, x),
            };
            data[y * width + x] = *pixel;
        }

        Self {
            width,
            height,
            data,
        }
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
