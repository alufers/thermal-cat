use std::{
    mem,
    ops::Index,
    sync::{mpsc, Arc},
    thread,
};

use eframe::epaint::{Color32, ColorImage, Pos2, Rect};
use nokhwa::{pixel_format::RgbFormat, Camera};

use crate::{
    camera_adapter::{infiray_p2_pro::InfirayP2ProAdapter, CameraAdapter},
    thermal_gradient::{ThermalGradient, ThermalGradientPoint, THERMAL_GRADIENTS},
};

pub struct ThermalCapturerResult {
    pub image: ColorImage,
    pub real_fps: f32,
    pub reported_fps: f32,
    pub range_min: f32,
    pub range_max: f32,
}

#[derive(Clone)]
pub struct ThermalCapturerRangeSettings {
    pub auto_range: bool,
    pub min_temp: f32,
    pub max_temp: f32,
}

pub type ThermalCapturerCallback = Arc<dyn Fn(ThermalCapturerResult) + Send + Sync>;

pub struct ThermalCapturer {
    ctx: Option<ThermalCapturerCtx>,
    cmd_writer: mpsc::Sender<ThermalCapturerCmd>,
}

enum ThermalCapturerCmd {
    SetGradient(ThermalGradient),
    SetRangeSettings(ThermalCapturerRangeSettings),
    Stop,
}

struct ThermalCapturerCtx {
    camera: Camera,
    callback: ThermalCapturerCallback,
    cmd_reader: mpsc::Receiver<ThermalCapturerCmd>,
    gradient: ThermalGradient,
    adapter: Arc<dyn CameraAdapter>,
    range_settings: ThermalCapturerRangeSettings,
}

///
/// ThermalCapturer runs in a background thread continuously capturing images from the camera,
/// And calling the callback function with the captured image.
impl ThermalCapturer {
    pub fn new(
        camera: Camera,
        adapter: Arc<dyn CameraAdapter>,
        callback: ThermalCapturerCallback,
    ) -> Self {
        let (cmd_writer, cmd_reader) = mpsc::channel();
        Self {
            ctx: Some(ThermalCapturerCtx {
                camera,
                adapter,
                callback,
                cmd_reader,
                gradient: THERMAL_GRADIENTS[0].clone(),
                range_settings: ThermalCapturerRangeSettings {
                    auto_range: true,
                    min_temp: 0.0,
                    max_temp: 100.0,
                },
            }),
            cmd_writer,
        }
    }

    //
    pub fn start(&mut self) {
        // move the camera out of self so we can use it into the thread
        let mut ctx = mem::replace(&mut self.ctx, None).unwrap();
        thread::spawn(move || {
            ctx.camera.open_stream().unwrap();

            let mut last_frame_time = std::time::Instant::now();
            let infiray = InfirayP2ProAdapter {};
            loop {
                last_frame_time = std::time::Instant::now();

                let thermal_data = infiray.capture_thermal_data(&mut ctx.camera).unwrap();

                let (mintemp_pos, maxtemp_pos) = thermal_data.get_min_max_pos();

                let mut min_temp = thermal_data.temperature_at(mintemp_pos.x, mintemp_pos.y);
                let mut max_temp = thermal_data.temperature_at(maxtemp_pos.x, maxtemp_pos.y);

                if !ctx.range_settings.auto_range {
                    min_temp = ctx.range_settings.min_temp;
                    max_temp = ctx.range_settings.max_temp;
                }

                let image = thermal_data.map_to_image(|temp| {
                    ctx.gradient
                        .get_color((temp - min_temp) / (max_temp - min_temp))
                });

                (ctx.callback)(ThermalCapturerResult {
                    image,
                    real_fps: 1.0 / last_frame_time.elapsed().as_secs_f32(),
                    reported_fps: ctx.camera.frame_rate() as f32,
                    range_min: min_temp,
                    range_max: max_temp,
                });
                match ctx.cmd_reader.try_recv() {
                    Ok(cmd) => match cmd {
                        ThermalCapturerCmd::Stop => {
                            ctx.camera.stop_stream().unwrap();
                            break;
                        }
                        ThermalCapturerCmd::SetGradient(gradient) => {
                            ctx.gradient = gradient;
                        }
                        ThermalCapturerCmd::SetRangeSettings(range_settings) => {
                            ctx.range_settings = range_settings;
                        }
                    },
                    Err(_) => {}
                }
            }
        });
    }

    pub fn set_gradient(&mut self, gradient: ThermalGradient) {
        self.cmd_writer
            .send(ThermalCapturerCmd::SetGradient(gradient))
            .unwrap();
    }
    pub fn set_range_settings(&mut self, range_settings: ThermalCapturerRangeSettings) {
        self.cmd_writer
            .send(ThermalCapturerCmd::SetRangeSettings(range_settings))
            .unwrap();
    }
}

impl Drop for ThermalCapturer {
    fn drop(&mut self) {
        self.cmd_writer.send(ThermalCapturerCmd::Stop).unwrap();
    }
}
