use std::sync::Arc;

use nokhwa::{utils::RequestedFormat, Camera, NokhwaError};
use once_cell::sync::Lazy;

use crate::thermal_data::ThermalData;

pub mod infiray_p2_pro;
pub mod thermal_master_p2;

pub static CAMERA_ADAPTERS: Lazy<Vec<Arc<dyn CameraAdapter>>> = Lazy::new(|| {
    vec![
        Arc::new(infiray_p2_pro::InfirayP2ProAdapter {}),
        Arc::new(thermal_master_p2::ThermanlMasterP2ProAdapter {}),
    ]
});
pub trait CameraAdapter: Send + Sync {
    ///
    /// Get friendly name of the camera model
    ///
    fn name(&self) -> String;

    ///
    /// Shorter name of the camera model for use in filenames
    ///
    fn short_name(&self) -> String;

    ///
    /// Get the USB PID/VID of the camera to match against
    ///
    fn usb_vid_pids(&self) -> Vec<(u16, u16)>;

    ///
    /// Get requested format for the camera
    ///
    fn requested_format(&self) -> RequestedFormat<'static>;

    ///
    /// Get the advertised temperature range of the camera
    /// (min, max)
    ///
    #[allow(dead_code)]
    fn temperature_range(&self) -> (f32, f32);

    ///
    /// Capture thermal data from a started camera stream
    ///
    fn capture_thermal_data(&self, cam: &mut Camera) -> Result<ThermalData, NokhwaError>;
}
