use std::path::PathBuf;

use anyhow::anyhow;
use chrono::{DateTime, Local};

use crate::{
    thermal_capturer::ThermalCapturerResult,
    types::media_formats::ImageFormat,
    util::{pathify_string, rgba8_to_rgb8},
};

use super::recorder::{Recorder, RecorderState};

pub struct ImageRecorder {
    // Params
    destination_folder: PathBuf,
    name_prefix: String,
    image_format: ImageFormat,

    // Output info
    output_file: Option<PathBuf>,
    curr_state: RecorderState,
}

impl ImageRecorder {
    pub fn new(
        destination_folder: PathBuf,
        name_prefix: String,
        image_format: ImageFormat,
    ) -> ImageRecorder {
        ImageRecorder {
            destination_folder,
            name_prefix,
            image_format,
            output_file: None,
            curr_state: RecorderState::Initial,
        }
    }
}

impl Recorder for ImageRecorder {
    fn start(
        &mut self,
        _params: super::recorder::RecorderStreamParams,
    ) -> Result<(), anyhow::Error> {
        self.curr_state = RecorderState::Recording;
        // Ignore params, we only capture a single image.
        Ok(())
    }

    fn process_result(&mut self, result: &ThermalCapturerResult) -> Result<(), anyhow::Error> {
        let image = &result.image;
        let rgba_img = image::RgbaImage::from_raw(
            image.width() as u32,
            image.height() as u32,
            image.as_raw().into(),
        )
        .ok_or(anyhow!("Failed to create image when saving snapshot"))?;

        // Convert to Rgb8, we don't need the alpha channel
        let img = rgba8_to_rgb8(rgba_img);

        std::fs::create_dir_all(self.destination_folder.clone())?;
        let current_local: DateTime<Local> = Local::now();

        let filename = format!(
            "{}_{}.{}",
            pathify_string(self.name_prefix.clone()),
            current_local.format("%Y-%m-%d_%H-%M-%S"),
            self.image_format.extension()
        );

        let save_path = self.destination_folder.join(PathBuf::from(filename));
        img.save(save_path.clone())?;
        self.output_file = Some(save_path);
        self.curr_state = RecorderState::Done;
        Ok(())
    }

    fn state(&self) -> RecorderState {
        self.curr_state
    }

    fn files_created(&self) -> Vec<PathBuf> {
        match &self.output_file {
            Some(file) => vec![file.clone()],
            None => vec![],
        }
    }

    fn stop(&mut self) -> Result<(), anyhow::Error> {
        self.curr_state = RecorderState::Done;
        Ok(())
    }
}
