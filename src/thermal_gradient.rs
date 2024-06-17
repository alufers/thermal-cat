use std::hash::{Hash, Hasher};

use eframe::epaint::{Color32, ColorImage};

use once_cell::sync::Lazy;
use uuid::{uuid, Uuid};

pub static THERMAL_GRADIENTS: Lazy<Vec<ThermalGradient>> = Lazy::new(|| {
    vec![
        ThermalGradient::new(
            uuid!("1d6233d0-7f8e-47c1-b092-0831cf587610"),
            "Cold-warm".to_string(),
            vec![
                ThermalGradientPoint::from_rgbv(0, 0, 0, 0.0 / 9.0),
                ThermalGradientPoint::from_rgbv(0, 0, 255, 1.0 / 9.0),
                ThermalGradientPoint::from_rgbv(0, 255, 255, 2.0 / 9.0),
                ThermalGradientPoint::from_rgbv(0, 255, 0, 3.0 / 9.0),
                ThermalGradientPoint::from_rgbv(255, 255, 0, 4.0 / 9.0),
                ThermalGradientPoint::from_rgbv(255, 128, 0, 5.0 / 9.0),
                ThermalGradientPoint::from_rgbv(255, 0, 0, 6.0 / 9.0),
                ThermalGradientPoint::from_rgbv(255, 0, 255, 7.0 / 9.0),
                ThermalGradientPoint::from_rgbv(255, 255, 255, 8.0 / 9.0),
            ],
        ),
        ThermalGradient::new(
            uuid!("6f2e8a5a-f38c-4347-9c23-2d9f2e7a4aae"),
            "Black to white".to_string(),
            vec![
                ThermalGradientPoint::from_rgbv(0, 0, 0, 0.0),
                ThermalGradientPoint::from_rgbv(255, 255, 255, 1.0),
            ],
        ),
        ThermalGradient::new(
            uuid!("07943b0b-0e36-463c-8895-5befe69c69d9"),
            "White to black".to_string(),
            vec![
                ThermalGradientPoint::from_rgbv(255, 255, 255, 0.0),
                ThermalGradientPoint::from_rgbv(0, 0, 0, 1.0),
            ],
        ),
    ]
});

#[derive(Clone)]
pub struct ThermalGradientPoint {
    color: Color32,
    pos: f32,
}

impl ThermalGradientPoint {
    pub fn from_rgbv(r: u8, g: u8, b: u8, pos: f32) -> Self {
        Self {
            color: Color32::from_rgb(r, g, b),
            pos,
        }
    }
}

impl Hash for ThermalGradientPoint {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.color.hash(state);
        self.pos.to_bits().hash(state);
    }
}

#[derive(Clone, Hash)]
pub struct ThermalGradient {
    ///
    /// UUID of the gradient (will be important when custom gradients are supported)
    ///
    pub uuid: Uuid,
    pub name: String,
    pub points: Vec<ThermalGradientPoint>,
}
impl ThermalGradient {
    pub fn new(uuid: Uuid, name: String, points: Vec<ThermalGradientPoint>) -> Self {
        let mut me = Self { uuid, name, points };
        me.points.sort_by(|a, b| a.pos.partial_cmp(&b.pos).unwrap());

        me
    }

    //
    // Sample the function at a given position.
    // The position is normalized to the range [0, 1].
    //
    pub fn get_color(&self, pos: f32) -> Color32 {
        if self.points.is_empty() {
            return Color32::from_rgb(0, 0, 0);
        }
        if self.points.len() == 1 {
            return self.points[0].color;
        }
        if pos < self.points[0].pos {
            return self.points[0].color;
        }
        if pos > self.points[self.points.len() - 1].pos {
            return self.points[self.points.len() - 1].color;
        }
        let mut i = 0;
        while i < self.points.len() - 1 {
            if pos >= self.points[i].pos && pos <= self.points[i + 1].pos {
                let t = (pos - self.points[i].pos) / (self.points[i + 1].pos - self.points[i].pos);
                return Color32::from_rgb(
                    (self.points[i].color.r() as f32 * (1.0 - t)
                        + self.points[i + 1].color.r() as f32 * t) as u8,
                    (self.points[i].color.g() as f32 * (1.0 - t)
                        + self.points[i + 1].color.g() as f32 * t) as u8,
                    (self.points[i].color.b() as f32 * (1.0 - t)
                        + self.points[i + 1].color.b() as f32 * t) as u8,
                );
            }
            i += 1;
        }
        Color32::from_rgb(0, 0, 0)
    }

    pub fn create_demo_image(&self, width: usize, height: usize) -> ColorImage {
        let mut pixels = vec![Color32::default(); width * height];

        for (i, pixel) in pixels.iter_mut().enumerate() {
            let x = i % width;
            let _y = i / width;

            let pos = x as f32 / width as f32;

            *pixel = self.get_color(pos);
        }

        ColorImage {
            pixels,
            size: [width, height],
        }
    }
}
