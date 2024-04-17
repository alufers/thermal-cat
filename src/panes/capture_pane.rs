use std::{
    cell::RefCell,
    path::{Path, PathBuf},
    rc::Rc,
};

use eframe::{
    egui::{self, Button},
    emath::Vec2b,
    epaint::Color32,
};
use egui_plot::{Bar, BarChart, Plot, VLine};

use crate::{
    pane_dispatcher::Pane, temperature::TemperatureUnit, thermal_capturer::SnapshotSettings,
    AppGlobalState,
};

pub struct CapturePane {
    global_state: Rc<RefCell<AppGlobalState>>,
}

impl CapturePane {
    pub fn new(global_state: Rc<RefCell<AppGlobalState>>) -> CapturePane {
        CapturePane { global_state }
    }
}

impl Pane for CapturePane {
    fn title(&self) -> egui::WidgetText {
        "Capture".into()
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        let global_state_clone = self.global_state.clone();
        let mut global_state = global_state_clone.as_ref().borrow_mut();

        ui.horizontal_centered(|ui| {
            if ui
                .add(Button::image_and_text(
                    egui::include_image!("../icons/camera.svg"),
                    "Snapshot",
                ))
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
                    })
                }
            }
        });
    }
}
