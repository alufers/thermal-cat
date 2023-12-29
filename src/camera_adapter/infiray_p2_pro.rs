use nokhwa::{
    pixel_format::RgbFormat,
    utils::{CameraFormat, FrameFormat, RequestedFormat, RequestedFormatType, Resolution},
    NokhwaError,
};

use crate::{temperature::Temp, thermal_data::ThermalData};

use super::CameraAdapter;

const IMAGE_WIDTH: u32 = 128;
const IMAGE_HEIGHT: u32 = 96/2;
const FRAME_RATE: u32 = 10;
pub struct InfirayP2ProAdapter {}

//
// Camera adapter for the Infiray P2 Pro
// See: https://www.infiray.com/p2-pro-thermal-camera-for-smartphone/
// The camera presents two video streams:
// - A 256x192 YUYV stream with greyscale representation of the thermal data (the scale changes depending on the temperature range)
// - A 256x348 YUYV stream with the same greyscale thermal data on top, and 256x156 uint16 thermal data underneath
//
// We are interested in the bottom part of the second stream, which contains the raw thermal data.
//
// The uint16 thermal data is a 256x192 array of 16-bit unsigned integers, representing the temperature in 1/64th's Kelvin
//
impl CameraAdapter for InfirayP2ProAdapter {
    fn name(&self) -> String {
        "Infiray P2 Pro".to_string()
    }

    fn requested_format(&self) -> nokhwa::utils::RequestedFormat<'static> {
        return RequestedFormat::new::<RgbFormat>(RequestedFormatType::Closest(CameraFormat::new(
            Resolution::new(IMAGE_WIDTH, IMAGE_HEIGHT * 2),
            FrameFormat::YUYV,
            FRAME_RATE,
        )));
    }

    fn temperature_range(&self) -> (f32, f32) {
        return (253.15, 873.15);
    }

    ///
    /// Capture and return thermal data
    fn capture_thermal_data(&self, cam: &mut nokhwa::Camera) -> Result<ThermalData, NokhwaError> {
        let frame_data: std::borrow::Cow<'_, [u8]> = cam.frame_raw()?;

        // crop to the bottom half of the frame, which contains the thermal data
        // We have IMAGE_WIDTH * IMAGE_HEIGHT times 2 bytes per pixel (YUYV)
        let thermal_data_buf = &frame_data[(IMAGE_WIDTH * IMAGE_HEIGHT * 2) as usize..];

        let u16_temperature_data = unsafe {
            std::slice::from_raw_parts(thermal_data_buf.as_ptr() as *const u16, IMAGE_WIDTH as usize * IMAGE_HEIGHT as usize)
        };

        Ok::<ThermalData, NokhwaError>(ThermalData::new(
            IMAGE_WIDTH as usize,
            IMAGE_HEIGHT as usize,
            u16_temperature_data
                .iter()
                .map(|&x| Temp::new(x as f32 / 64.0))
                .collect(),
        ))
    }

    fn usb_vid_pid(&self) -> (u16, u16) {
        // Bus 001 Device 061: ID 0bda:5830 Realtek Semiconductor Corp. USB Camera
        //
        return (0xcafe, 0x4020);
    }
}
