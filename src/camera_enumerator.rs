use std::error::Error;

use nokhwa::{utils::CameraInfo, native_api_backend, query, NokhwaError};


#[derive(Debug,Clone)]
pub struct EnumerationError {
    message: String,
}

impl std::fmt::Display for EnumerationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}",self.message)
    }
}

impl Error for EnumerationError {}

//
// Contains extended information about a camera, including the USB PID/VID.
//
pub struct EnumeratedCamera {
    pub info: CameraInfo,
    pub usb_pid_vid: Option<(u16, u16)>,
}

impl EnumeratedCamera {
    pub fn friendly_name(&self) -> String {
        let mut name = self.info.human_name().clone();
        if let Some((pid, vid)) = self.usb_pid_vid {
            name.push_str(&format!(" (USB {:04x}:{:04x})", vid, pid));
        }
        return name;
    }
}

pub fn enumerate_cameras() -> Result<Vec<EnumeratedCamera>, Box<dyn Error>> {
    let backend = native_api_backend().ok_or(
        EnumerationError {
            message: "Failed to initialize Nokhwa backend".to_string(),
        }
    )?;

    let nokhwa_cameras = query(backend)?;

    return Ok::<Vec<EnumeratedCamera>, Box<dyn Error>>(nokhwa_cameras.into_iter().map(|info| {
        
        EnumeratedCamera {
            info,
            usb_pid_vid: None,
        }
    }).collect());
}
