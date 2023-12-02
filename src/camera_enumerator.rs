use std::{error::Error, sync::Arc};

use eframe::{
    egui::{TextFormat},
    epaint::{text::LayoutJob, Color32, FontFamily, FontId},
};
use nokhwa::{native_api_backend, query, utils::CameraInfo};

use once_cell::sync::Lazy;
use regex::Regex;

#[derive(Debug, Clone)]
pub struct EnumerationError {
    message: String,
}

impl std::fmt::Display for EnumerationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for EnumerationError {}

//
// Contains extended information about a camera, including the USB PID/VID.
//
pub struct EnumeratedCamera {
    pub info: CameraInfo,
    pub usb_vid_pid: Option<(u16, u16)>,
    pub adapter: Option<Arc<dyn crate::camera_adapter::CameraAdapter>>,
}

impl EnumeratedCamera {
    pub fn rich_text_name(&self, shorten: bool) -> LayoutJob {
        let mut job = LayoutJob::default();
        job.append(
            &self.info.human_name().clone(),
            0.0,
            TextFormat {
                color: Color32::WHITE,
                ..Default::default()
            },
        );
        if let Some(adapter) = &self.adapter {
            job.append(
                &format!("\n({})", adapter.name()),
                0.0,
                TextFormat {
                    color: Color32::GREEN,
                    ..Default::default()
                },
            );
        }
        if let Some((vid, pid)) = self.usb_vid_pid {
            if !shorten {
                job.append(
                    &format!("\n(USB {:04x}:{:04x})", vid, pid),
                    0.0,
                    TextFormat {
                        color: Color32::GRAY,
                        font_id: FontId::new(14.0, FontFamily::Monospace),
                        ..Default::default()
                    },
                );
            }
        }

        return job;
    }
}

pub fn enumerate_cameras() -> Result<Vec<EnumeratedCamera>, Box<dyn Error>> {
    let backend = native_api_backend().ok_or(EnumerationError {
        message: "Failed to initialize Nokhwa backend".to_string(),
    })?;

    let nokhwa_cameras = query(backend)?;

    return Ok::<Vec<EnumeratedCamera>, Box<dyn Error>>(
        nokhwa_cameras
            .into_iter()
            .map(|info| {
                let usb_vid_pid = get_vid_pid_for_camera(&info);
                let adapter = crate::camera_adapter::CAMERA_ADAPTERS
                    .iter()
                    .find(|adapter| {
                        if let Some((vid, pid)) = usb_vid_pid {
                            return adapter.usb_vid_pid() == (vid, pid);
                        } else {
                            return false;
                        }
                    })
                    .map(|adapter| adapter.clone());
                EnumeratedCamera {
                    info,
                    usb_vid_pid,
                    adapter,
                }
            })
            .collect(),
    );
}

static DEV_VIDEO_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"/dev/video(\d+)").unwrap());

static UEVENT_PRODUCT_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"PRODUCT=(\w+)/(\w+)/(\w+)").unwrap());

#[cfg(target_os = "linux")]
fn get_vid_pid_for_camera(info: &CameraInfo) -> Option<(u16, u16)> {
    // extract /dev/videoX from the device description

    use std::fs;

    let descr = info.description().to_string();

    DEV_VIDEO_REGEX
        .captures(&descr)
        .and_then(|captures| {
            let dev_num = captures.get(1).unwrap().as_str().parse::<u16>().unwrap();
            let uevent_path = format!("/sys/class/video4linux/video{}/device/uevent", dev_num);
            return fs::read_to_string(uevent_path).ok();
        })
        .and_then(|uevent_contents| {
            UEVENT_PRODUCT_REGEX
                .captures(&uevent_contents)
                .and_then(|captures| {
                    let vid = u16::from_str_radix(captures.get(1).unwrap().as_str(), 16).unwrap();
                    let pid = u16::from_str_radix(captures.get(2).unwrap().as_str(), 16).unwrap();
                    return Some((vid, pid));
                })
        })
}

#[cfg(not(target_os = "linux"))]
fn get_vid_pid_for_camera(info: &CameraInfo) -> Option<(u16, u16)> {
    None
}
