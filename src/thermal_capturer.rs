use std::{mem, sync::{mpsc, Arc}, thread};

use eframe::epaint::ColorImage;
use nokhwa::{pixel_format::RgbFormat, Camera};



pub type ThermalCapturerCallback = Arc<dyn Fn(ColorImage) + Send + Sync>;

pub struct ThermalCapturer {
    ctx: Option<ThermalCapturerCtx>,
    cmd_writer: mpsc::Sender<ThermalCapturerCmd>,
}

enum ThermalCapturerCmd {
    Stop,
}

struct ThermalCapturerCtx {
    camera: Camera,
    callback: ThermalCapturerCallback,
    cmd_reader: mpsc::Receiver<ThermalCapturerCmd>,
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
            loop {
                let frame = ctx.camera.frame().unwrap();
                let decoded = frame.decode_image::<RgbFormat>().unwrap();
                let image = ColorImage::from_rgb(
                    [decoded.width() as usize, decoded.height() as usize],
                    decoded.as_raw(),
                );
                (ctx.callback)(image);
                match ctx.cmd_reader.try_recv() {
                    Ok(cmd) => match cmd {
                        ThermalCapturerCmd::Stop => {
                            ctx.camera.stop_stream().unwrap();
                            break;
                        }
                    },
                    Err(_) => {}
                }
            }
        });
    }
}

impl Drop for ThermalCapturer {
    fn drop(&mut self) {
        self.cmd_writer.send(ThermalCapturerCmd::Stop).unwrap();
    }
}
