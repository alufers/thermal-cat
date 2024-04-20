use std::{
    cell::RefCell,
    collections::VecDeque,
    path::{Path, PathBuf},
    rc::Rc,
};

use eframe::egui::{self, scroll_area::ScrollBarVisibility, Align, Button, Image, Layout, Vec2};

use crate::{
    pane_dispatcher::Pane,
    record_video::VideoRecordingSettings,
    thermal_capturer::SnapshotSettings,
    types::media_formats::{all_media_file_extensions, ImageFormat, VideoFormat},
    AppGlobalState,
};

#[derive(Debug, Clone)]
pub struct GalleryElement {
    pub path: PathBuf,
    pub created_at: std::time::SystemTime,
}

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
        if let Err(err) = self.init_gallery() {
            eprintln!("Failed to initialize gallery: {:?}", err);
        }
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
                        let captures_dir = global_state
                            .prefs
                            .as_ref()
                            .map(|prefs| prefs.captures_directory.clone())
                            .unwrap_or("./".to_string());

                        if let Some(thermal_capturer) = global_state.thermal_capturer_inst.as_mut()
                        {
                            thermal_capturer.stop_video_recording()
                        }
                    }
                } else {
                    if ui
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

                        if let Some(thermal_capturer) = global_state.thermal_capturer_inst.as_mut()
                        {
                            thermal_capturer.start_video_recording(VideoRecordingSettings {
                                output_path: PathBuf::from(captures_dir),
                                format: self.video_format,
                                height: 0,
                                width: 0,
                            })
                        }
                    }
                }
            });
        });

        ui.separator();
        ui.label("Gallery");
        egui::ScrollArea::vertical()
            .scroll_bar_visibility(ScrollBarVisibility::AlwaysVisible)
            .show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    for elem in &global_state.gallery {
                        let base_name = elem.path.file_name().unwrap().to_string_lossy();

                        ui.add(
                            Image::new("file://".to_string() + elem.path.to_str().unwrap())
                                .fit_to_exact_size(Vec2::new(150.0, 100.0))
                                .maintain_aspect_ratio(true),
                        );
                        ui.label(base_name);
                        ui.add_space(2.0);
                    }
                });
            });
    }
}

impl CapturePane {
    // Loads files from the captures directory and initializes the gallery
    fn init_gallery(&mut self) -> Result<(), anyhow::Error> {
        let global_state_clone = self.global_state.clone();
        let mut global_state = global_state_clone.as_ref().borrow_mut();

        if global_state.did_init_gallery {
            return Ok(());
        }
        global_state.did_init_gallery = true;

        let captures_dir = global_state
            .prefs
            .as_ref()
            .map(|prefs| prefs.captures_directory.clone())
            .unwrap_or("./".to_string());

        let captures_dir = Path::new(&captures_dir);

        if !captures_dir.exists() {
            return Ok(());
        }
        let all_known_extensions = all_media_file_extensions();
        let mut gallery_vec: Vec<GalleryElement> = captures_dir
            .read_dir()?
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                let ext = path.extension()?.to_string_lossy().to_string();
                if path.is_file() && all_known_extensions.contains(&ext) {
                    let metadata = entry.metadata().ok()?;

                    Some(GalleryElement {
                        path,
                        created_at: metadata.created().ok()?,
                    })
                } else {
                    None
                }
            })
            .collect();

        gallery_vec.sort_by(|a, b| a.created_at.cmp(&b.created_at));

        // Limit the vector to the last 20 items
        let last_items = gallery_vec.iter().rev().take(20).collect::<Vec<_>>();

        global_state.gallery = VecDeque::with_capacity(20);
        for item in last_items {
            global_state.gallery.push_back(item.clone());
        }

        Ok(())
    }
}
