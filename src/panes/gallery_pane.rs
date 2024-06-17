use std::{
    cell::RefCell,
    collections::VecDeque,
    path::{Path, PathBuf},
    rc::Rc,
};

use eframe::egui::{
    self, scroll_area::ScrollBarVisibility, Align, Button, Color32, Image, Layout, Ui, Vec2,
};

use crate::{
    pane_dispatcher::Pane,
    thermal_capturer::{SnapshotSettings, StartVideoRecordingSettings},
    types::media_formats::{all_media_file_extensions, ImageFormat, VideoFormat},
    AppGlobalState,
};

#[derive(Debug, Clone)]
pub struct GalleryElement {
    pub path: PathBuf,
    pub created_at: std::time::SystemTime,
}

pub struct GalleryPane {
    global_state: Rc<RefCell<AppGlobalState>>,
}

impl GalleryPane {
    pub fn new(global_state: Rc<RefCell<AppGlobalState>>) -> GalleryPane {
        GalleryPane { global_state }
    }
}

impl Pane for GalleryPane {
    fn title(&self) -> egui::WidgetText {
        "Gallery".into()
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        if let Err(err) = self.init_gallery() {
            eprintln!("Failed to initialize gallery: {:?}", err);
        }
        let global_state_clone = self.global_state.clone();
        let mut global_state = global_state_clone.as_ref().borrow_mut();

        // Width of each element in the gallery
        const ELEM_WIDTH: f32 = 150.0;

        egui::ScrollArea::vertical()
            .scroll_bar_visibility(ScrollBarVisibility::AlwaysVisible)
            .show(ui, |ui| {
                let available_width = ui.available_width();
                ui.set_width(available_width);
                ui.with_layout(
                    Layout::left_to_right(Align::Min)
                        .with_cross_justify(false)
                        .with_main_wrap(true),
                    |ui| {
                        for elem in &global_state.gallery {
                            let base_name = elem.path.file_name().unwrap().to_string_lossy();

                            // Hacky justification
                            let container_width =
                                (available_width) / (available_width / ELEM_WIDTH).floor() - 8.0;
                            println!("container_width: {}", container_width);
                            ui.add_sized(Vec2::new(container_width, 110.0), |ui: &mut Ui| {
                                ui.vertical_centered(|ui| {
                                    ui.add(
                                        Image::new(
                                            "file://".to_string() + elem.path.to_str().unwrap(),
                                        )
                                        .fit_to_exact_size(Vec2::new(ELEM_WIDTH, 100.0))
                                        .maintain_aspect_ratio(true),
                                    );
                                    ui.label(base_name);
                                    ui.add_space(2.0);
                                })
                                .response
                            });
                        }
                    },
                );
            });
    }
}

impl GalleryPane {
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

                // Only generate thumbnails for:
                // - files
                // - files with known extensions
                // - files that are at least 256 bytes in size, to avoid generating thumbnails for empty and corrupt files
                let size_ok = entry
                    .metadata()
                    .ok()
                    .map(|metadata| metadata.len() >= 256)
                    .unwrap_or(false);
                if path.is_file() && all_known_extensions.contains(&ext) && size_ok {
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
