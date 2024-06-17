use std::{cell::RefCell, path::PathBuf, rc::Rc};

use eframe::egui::{self, Align, Button, Color32, Layout, Vec2};

use crate::{
    pane_dispatcher::Pane,
    thermal_capturer::{SnapshotSettings, StartVideoRecordingSettings},
    types::media_formats::{ImageFormat, VideoFormat},
    AppGlobalState,
};

pub struct CapturePane {
    global_state: Rc<RefCell<AppGlobalState>>,
    snapshot_format: ImageFormat,
    video_format: VideoFormat,
}

impl CapturePane {
    pub fn new(global_state: Rc<RefCell<AppGlobalState>>) -> CapturePane {
        CapturePane {
            global_state,
            snapshot_format: ImageFormat::Png,
            video_format: VideoFormat::MP4_H264,
        }
    }
}

impl Pane for CapturePane {
    fn title(&self) -> egui::WidgetText {
        "Capture".into()
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        let global_state_clone = self.global_state.clone();
        let mut global_state = global_state_clone.as_ref().borrow_mut();

        let available_width = ui.available_width();
        ui.with_layout(Layout::left_to_right(egui::Align::Min), |ui| {
            ui.with_layout(Layout::top_down_justified(Align::Min), |ui| {
                ui.set_max_width(available_width / 2.0 - 5.0);
                ImageFormat::egui_combo_box(
                    ui,
                    "capture_pane_snapshot_format",
                    &mut self.snapshot_format,
                    available_width / 2.0 - 5.0,
                );

                if ui
                    .add(
                        Button::image_and_text(
                            egui::include_image!("../icons/camera.svg"),
                            "Snapshot",
                        )
                        .min_size(Vec2::new(0.0, 25.0)),
                    )
                    .clicked()
                {
                    let captures_dir = global_state
                        .prefs
                        .as_ref()
                        .map(|prefs| prefs.captures_directory.clone())
                        .unwrap_or("./".to_string());

                    if let Some(thermal_capturer) = global_state.thermal_capturer_inst.as_mut() {
                        thermal_capturer.take_snapshot(SnapshotSettings {
                            dir_path: PathBuf::from(captures_dir),
                            image_format: self.snapshot_format,
                        })
                    }
                }
            });

            ui.with_layout(Layout::top_down_justified(Align::Min), |ui| {
                VideoFormat::egui_combo_box(
                    ui,
                    "capture_pane_video_format",
                    &mut self.video_format,
                    available_width / 2.0 - 5.0,
                );
                let is_recording = global_state
                    .last_thermal_capturer_result
                    .as_ref()
                    .map(|res| res.is_recording_video)
                    .unwrap_or(false);

                if is_recording {
                    ui.scope(|ui| {
                        ui.style_mut().visuals.widgets.inactive.weak_bg_fill =
                            Color32::from_hex("#e61b29").unwrap();
                        ui.style_mut().visuals.widgets.inactive.fg_stroke.color =
                            Color32::from_hex("#ffffff").unwrap();
                        ui.style_mut().visuals.widgets.hovered.weak_bg_fill =
                            Color32::from_hex("#e61b29").unwrap();
                        if ui
                            .add(
                                Button::image_and_text(
                                    egui::include_image!("../icons/video.svg"),
                                    "Stop recording",
                                )
                                .min_size(Vec2::new(available_width / 2.0 - 5.0, 25.0)),
                            )
                            .clicked()
                        {
                            if let Some(thermal_capturer) =
                                global_state.thermal_capturer_inst.as_mut()
                            {
                                thermal_capturer.stop_video_recording()
                            }
                        }
                    });
                } else if ui
                    .add(
                        Button::image_and_text(
                            egui::include_image!("../icons/video.svg"),
                            "Record video",
                        )
                        .min_size(Vec2::new(available_width / 2.0 - 5.0, 25.0)),
                    )
                    .clicked()
                {
                    let captures_dir = global_state
                        .prefs
                        .as_ref()
                        .map(|prefs| prefs.captures_directory.clone())
                        .unwrap_or("./".to_string());

                    if let Some(thermal_capturer) = global_state.thermal_capturer_inst.as_mut() {
                        thermal_capturer.start_video_recording(StartVideoRecordingSettings {
                            output_dir: PathBuf::from(captures_dir),
                            format: self.video_format,
                        })
                    }
                }
            });
        });
    }
}
