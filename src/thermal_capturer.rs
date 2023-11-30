use std::{thread, mem};

use eframe::epaint::ColorImage;
use nokhwa::{Camera, pixel_format::RgbFormat};

struct ThermalCapturerCtx {
    camera: Camera,
    callback: fn(ColorImage),
}

struct ThermalCapturer {
    ctx: Option<ThermalCapturerCtx>,
}

///
/// ThermalCapturer runs in a background thread continuously capturing images from the camera,
/// And calling the callback function with the captured image.
impl ThermalCapturer {
    fn new(camera: Camera, callback: fn(ColorImage)) -> Self {
        Self {
            ctx: Some(ThermalCapturerCtx {
                camera,
                callback,
            }),
        }
    }
    fn start(&mut self) {
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
            }
        });
    }
}
