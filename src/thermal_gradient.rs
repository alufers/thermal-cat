use eframe::epaint::{Color32, ColorImage};

use once_cell::sync::Lazy;



pub static THERMAL_GRADIENTS: Lazy<Vec<ThermalGradient>> = Lazy::new(|| {
    vec![
        ThermalGradient::new("Cold-warm".to_string(), vec![
            ThermalGradientPoint::from_rgbv(0, 0, 0, 0.0),
            ThermalGradientPoint::from_rgbv(0, 0, 255, 0.21),
            ThermalGradientPoint::from_rgbv(0, 255, 255, 0.24),
            ThermalGradientPoint::from_rgbv(0, 255, 0, 0.26),
            ThermalGradientPoint::from_rgbv(255, 255, 0, 0.29),
            ThermalGradientPoint::from_rgbv(255, 128, 0, 0.32),
            ThermalGradientPoint::from_rgbv(255, 0, 0, 0.35),
            ThermalGradientPoint::from_rgbv(255, 0, 255, 0.71),
            ThermalGradientPoint::from_rgbv(255, 255, 255, 1.0),
        ]),
        ThermalGradient::new("Black to white".to_string(), vec![
            ThermalGradientPoint::from_rgbv(0, 0, 0, 0.0),
            ThermalGradientPoint::from_rgbv(255, 255, 255, 1.0),
        ]),
        ThermalGradient::new("White to black".to_string(), vec![
            ThermalGradientPoint::from_rgbv(255, 255, 255, 0.0),
            ThermalGradientPoint::from_rgbv(0, 0, 0, 1.0),
        ]),
    ]
});

#[derive(Clone)]
pub struct ThermalGradientPoint {
    color: Color32,
    pos: f32,
}

impl ThermalGradientPoint {
    pub fn new(color: Color32, pos: f32) -> Self {
        Self { color, pos }
    }

    pub fn from_rgbv(r: u8, g: u8, b: u8, pos: f32) -> Self {
        Self {
            color: Color32::from_rgb(r, g, b),
            pos: pos,
        }
    }
}

#[derive(Clone)]
pub struct ThermalGradient {
    pub name: String,
    pub points: Vec<ThermalGradientPoint>,
}
impl ThermalGradient {
    pub fn new(name: String, points: Vec<ThermalGradientPoint>) -> Self {
        let mut me = Self { name, points };
        me.points
            .sort_by(|a, b| a.pos.partial_cmp(&b.pos).unwrap());

        me
    }

    //
    // Sample the function at a given position.
    // The position is normalized to the range [0, 1].
    //
    pub fn get_color(&self, pos: f32) -> Color32 {
        if self.points.len() == 0 {
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
                let t =
                    (pos - self.points[i].pos) / (self.points[i + 1].pos - self.points[i].pos);
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
        return Color32::from_rgb(0, 0, 0);
    }

    pub fn create_demo_image(&self, width: usize, height: usize) -> ColorImage {
        let mut pixels = vec![Color32::default(); width * height];
        
        for (i, pixel) in pixels.iter_mut().enumerate() {
            let x = i % width;
            let y = i / width;

            let pos = x as f32 / width as f32;

            *pixel = self.get_color(pos);
        }

        ColorImage {
            pixels,
            size: [width, height],
        }
    }
}
