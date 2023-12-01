use std::error::Error;

use nokhwa::{utils::RequestedFormat, Camera, NokhwaError};

use crate::thermal_data::ThermalData;

pub mod infiray_p2_pro;

pub trait CameraAdapter {
    fn new() -> Self;
    ///
    /// Get friendly name of the camera model
    ///
    fn name(&self) -> String;

    ///
    /// Get requested format for the camera
    ///
    fn requested_format(&self) -> RequestedFormat<'static>;

    ///
    /// Get the advertised temperature range of the camera
    /// (min, max)
    /// 
    fn temperature_range(&self) -> (f32, f32);

    ///
    /// Capture thermal data from a started camera stream
    ///
    fn capture_thermal_data(&self, cam: &mut Camera) -> Result<ThermalData, impl Error>;
    
}
