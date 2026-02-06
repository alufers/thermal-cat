use nokhwa::{
    pixel_format::RgbFormat,
    utils::{CameraFormat, FrameFormat, RequestedFormat, RequestedFormatType, Resolution},
    NokhwaError,
};

use crate::{temperature::Temp, thermal_data::ThermalData};

use super::CameraAdapter;

const IMAGE_WIDTH: u32 = 256;
const IMAGE_HEIGHT: u32 = 192;
const OFFSET: u32 = 2;
const COMBINDED_IMAGE_HEIGHT: u32 = IMAGE_HEIGHT * 2 + OFFSET;

pub struct ThermanlMasterP2ProAdapter {}

//
// Camera adapter for the Thermal Master P2
// https://thermalmaster.com/de-de/products/thermal-master-p2
// The camera presents two video streams:
// - A 256x192 YUYV stream with greyscale representation of the thermal data (the scale changes depending on the temperature range)
// - A 256x386 YUYV stream with the same greyscale thermal data on top, a 2 row offset and 256x192 uint16 thermal data underneath
//
// We are interested in the bottom part of the second stream, which contains the raw thermal data.
//
// The uint16 thermal data is a 256x192 array of 16-bit unsigned integers, representing the temperature in 1/64th's Kelvin
//
impl CameraAdapter for ThermanlMasterP2ProAdapter {
    fn name(&self) -> String {
        "Thermal Master P2".to_string()
    }

    fn short_name(&self) -> String {
        "P2".to_string()
    }

    fn requested_format(&self) -> nokhwa::utils::RequestedFormat<'static> {
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::Closest(CameraFormat::new(
            Resolution::new(IMAGE_WIDTH, COMBINDED_IMAGE_HEIGHT),
            FrameFormat::YUYV,
            25,
        )))
    }

    fn temperature_range(&self) -> (f32, f32) {
        (253.15, 873.15)
    }

    ///
    /// Capture and return thermal data
    fn capture_thermal_data(&self, cam: &mut nokhwa::Camera) -> Result<ThermalData, NokhwaError> {
        let frame_data: std::borrow::Cow<'_, [u8]> = cam.frame_raw()?;

        // crop to the bottom half of the frame, which contains the thermal data
        // We have IMAGE_WIDTH * IMAGE_HEIGHT times 2 bytes per pixel (YUYV)
        let thermal_data_buf =
            &frame_data[((IMAGE_WIDTH * (IMAGE_HEIGHT + OFFSET)) * 2) as usize..];

        let u16_temperature_data = unsafe {
            std::slice::from_raw_parts(
                thermal_data_buf.as_ptr() as *const u16,
                (IMAGE_WIDTH * IMAGE_HEIGHT) as usize,
            )
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

    fn usb_vid_pids(&self) -> Vec<(u16, u16)> {
        // Bus 001 Device 003: ID 3474:4281 Raysentek Co.,Ltd Camera
        vec![(0x3474, 0x4281)]
    }
}
