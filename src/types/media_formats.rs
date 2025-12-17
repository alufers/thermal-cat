use std::hash::Hash;

use eframe::egui::{ComboBox, Ui};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, EnumIter)]
pub enum ImageFormat {
    Jpeg,
    Png,
}

impl ImageFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            ImageFormat::Jpeg => "jpeg",
            ImageFormat::Png => "png",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            ImageFormat::Jpeg => "JPEG",
            ImageFormat::Png => "PNG",
        }
    }

    pub fn egui_combo_box(ui: &mut Ui, id_source: impl Hash, value: &mut Self, width: f32) {
        ComboBox::from_id_salt(id_source)
            .selected_text(value.name())
            .width(width)
            .show_ui(ui, |ui| {
                for format in Self::iter() {
                    ui.selectable_value(value, format, format.name());
                }
            });
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, EnumIter)]
#[allow(non_camel_case_types)]
pub enum VideoFormat {
    MP4_H264,
    WEBM_VP9,
    MKV_VP9,
}

impl VideoFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            VideoFormat::MP4_H264 => "mp4",
            VideoFormat::WEBM_VP9 => "webm",
            VideoFormat::MKV_VP9 => "mkv",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            VideoFormat::MP4_H264 => "MP4 (H.264)",
            VideoFormat::WEBM_VP9 => "WebM (VP9)",
            VideoFormat::MKV_VP9 => "MKV (VP9)",
        }
    }

    pub fn egui_combo_box(ui: &mut Ui, id_source: impl Hash, value: &mut Self, width: f32) {
        ComboBox::from_id_salt(id_source)
            .selected_text(value.name())
            .width(width)
            .show_ui(ui, |ui| {
                for format in Self::iter() {
                    ui.selectable_value(value, format, format.name());
                }
            });
    }
}

pub fn all_media_file_extensions() -> Vec<String> {
    let mut extensions = vec![];
    for format in ImageFormat::iter() {
        extensions.push(format.extension().to_string());
    }
    for format in VideoFormat::iter() {
        extensions.push(format.extension().to_string());
    }
    extensions
}
