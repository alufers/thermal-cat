use eframe::epaint::{Color32, ColorImage};

use once_cell::sync::Lazy;



pub static THERMAL_GRADIENTS: Lazy<Vec<ThermalGradient>> = Lazy::new(|| {
    vec![
        ThermalGradient::new("Cold-warm".to_string(), vec![
            ThermalGradientPoint::from_rgbtemp(0, 0, 0, -20),
            ThermalGradientPoint::from_rgbtemp(0, 0, 255, 15),
            ThermalGradientPoint::from_rgbtemp(0, 255, 255, 20),
            ThermalGradientPoint::from_rgbtemp(0, 255, 0, 25),
            ThermalGradientPoint::from_rgbtemp(255, 255, 0, 30),
            ThermalGradientPoint::from_rgbtemp(255, 128, 0, 35),
            ThermalGradientPoint::from_rgbtemp(255, 0, 0, 40),
            ThermalGradientPoint::from_rgbtemp(255, 0, 255, 100),
            ThermalGradientPoint::from_rgbtemp(255, 255, 255, 150),
        ]),
        ThermalGradient::new("Black to white".to_string(), vec![
            ThermalGradientPoint::from_rgbtemp(0, 0, 0, -20),
            ThermalGradientPoint::from_rgbtemp(255, 255, 255, 180),
        ]),
        ThermalGradient::new("White to black".to_string(), vec![
            ThermalGradientPoint::from_rgbtemp(255, 255, 255, -20),
            ThermalGradientPoint::from_rgbtemp(0, 0, 0, 180),
        ]),
    ]
});

#[derive(Clone)]
pub struct ThermalGradientPoint {
    color: Color32,
    temp: f32,
}

impl ThermalGradientPoint {
    pub fn new(color: Color32, temp: f32) -> Self {
        Self { color, temp }
    }

    pub fn from_rgbtemp(r: u8, g: u8, b: u8, temp: i16) -> Self {
        Self {
            color: Color32::from_rgb(r, g, b),
            temp: temp as f32,
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
            .sort_by(|a, b| a.temp.partial_cmp(&b.temp).unwrap());

        me
    }

    pub fn get_color(&self, temp: f32) -> Color32 {
        if self.points.len() == 0 {
            return Color32::from_rgb(0, 0, 0);
        }
        if self.points.len() == 1 {
            return self.points[0].color;
        }
        if temp < self.points[0].temp {
            return self.points[0].color;
        }
        if temp > self.points[self.points.len() - 1].temp {
            return self.points[self.points.len() - 1].color;
        }
        let mut i = 0;
        while i < self.points.len() - 1 {
            if temp >= self.points[i].temp && temp <= self.points[i + 1].temp {
                let t =
                    (temp - self.points[i].temp) / (self.points[i + 1].temp - self.points[i].temp);
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
        let temp_start = -10.0;
        let temp_end = 150.0;
        for (i, pixel) in pixels.iter_mut().enumerate() {
            let x = i % width;
            let y = i / width;

            let temp = temp_start + (temp_end - temp_start) * (x as f32 / width as f32);

            *pixel = self.get_color(temp);
        }

        ColorImage {
            pixels,
            size: [width, height],
        }
    }
}
