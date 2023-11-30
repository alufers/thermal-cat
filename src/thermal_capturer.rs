
use std::{
    mem,
    ops::Index,
    sync::{mpsc, Arc},
    thread,
};

use eframe::epaint::{Color32, ColorImage, Pos2, Rect};
use nokhwa::{pixel_format::RgbFormat, Camera};

use crate::thermal_gradient::{ThermalGradient, ThermalGradientPoint, THERMAL_GRADIENT_DEFAULT, THERMAL_GRADIENTS};

pub struct ThermalCapturerResult {
    pub image: ColorImage,
    pub real_fps: f32,
    pub reported_fps: f32,
}

pub type ThermalCapturerCallback = Arc<dyn Fn(ThermalCapturerResult) + Send + Sync>;

pub struct ThermalCapturer {
    ctx: Option<ThermalCapturerCtx>,
    cmd_writer: mpsc::Sender<ThermalCapturerCmd>,
}

enum ThermalCapturerCmd {
    SetGradient(ThermalGradient),
    Stop,
}

struct ThermalCapturerCtx {
    camera: Camera,
    callback: ThermalCapturerCallback,
    cmd_reader: mpsc::Receiver<ThermalCapturerCmd>,
    gradient: ThermalGradient,
}

///
/// ThermalCapturer runs in a background thread continuously capturing images from the camera,
/// And calling the callback function with the captured image.
impl ThermalCapturer {
    pub fn new(camera: Camera, callback: ThermalCapturerCallback) -> Self {
        let (cmd_writer, cmd_reader) = mpsc::channel();
        Self {
            ctx: Some(ThermalCapturerCtx {
                camera,
                callback: callback,
                cmd_reader,
                gradient: THERMAL_GRADIENTS[0].clone(),
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
            loop {
                last_frame_time = std::time::Instant::now();
                let frame = ctx.camera.frame().unwrap();

                let frame_data = frame.buffer();

                let thermal_data_buf = &frame_data[256 * 192 * 2..];

                let thermal_data = unsafe {
                    std::slice::from_raw_parts(thermal_data_buf.as_ptr() as *const u16, 256 * 192)
                };

                let mut color_image_pixels = vec![Color32::default(); 256 * 192];

                for (i, pixel) in thermal_data.iter().enumerate() {
                   
                    color_image_pixels[i] = ctx.gradient.get_color((*pixel as f32) / 64.0 - 273.15);
                }

                let image = ColorImage {
                    pixels: color_image_pixels,
                    size: [256, 192],
                };

                (ctx.callback)(ThermalCapturerResult {
                    image,
                    real_fps: 1.0 / last_frame_time.elapsed().as_secs_f32(),
                    reported_fps: ctx.camera.frame_rate() as f32,
                });
                match ctx.cmd_reader.try_recv() {
                    Ok(cmd) => match cmd {
                        ThermalCapturerCmd::Stop => {
                            ctx.camera.stop_stream().unwrap();
                            break;
                        },
                        ThermalCapturerCmd::SetGradient(gradient) => {
                            ctx.gradient = gradient;
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
}

impl Drop for ThermalCapturer {
    fn drop(&mut self) {
        self.cmd_writer.send(ThermalCapturerCmd::Stop).unwrap();
    }
}
