use std::{cell::RefCell, rc::Rc};

use crate::{
    pane_dispatcher::Pane, temperature::TemperatureUnit, user_preferences::UserPreferences,
    AppGlobalState,
};
use anyhow::Context;
use eframe::egui::{self, Grid};
use log::error;
use strum::IntoEnumIterator;

pub struct UserPreferencesPane {
    local_user_preferences: Option<UserPreferences>,
    global_state: Rc<RefCell<AppGlobalState>>,
}

impl UserPreferencesPane {
    pub fn new(global_state: Rc<RefCell<AppGlobalState>>) -> Self {
        let copied_prefs = global_state.as_ref().borrow().prefs.clone();
        Self {
            local_user_preferences: copied_prefs,
            global_state,
        }
    }
}

impl Pane for UserPreferencesPane {
    fn title(&self) -> egui::WidgetText {
        "User Preferences".into()
    }
    fn ui(&mut self, ui: &mut egui::Ui) {
        let global_state_clone = self.global_state.clone();
        let mut global_state = global_state_clone.as_ref().borrow_mut();
        if self.local_user_preferences.is_some() {
            ui.heading("User Preferences");
            ui.separator();
            Grid::new("gradient_grid")
                .num_columns(2)
                .spacing([10.0, 10.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label("Name");
                    ui.label("Value");
                    ui.end_row();
                    let edited_prefs = self.local_user_preferences.as_mut().unwrap();

                    ui.label("Temperature Unit");
                    egui::ComboBox::from_label("")
                        .selected_text(edited_prefs.temperature_unit.to_string())
                        .show_ui(ui, |ui| {
                            for unit in TemperatureUnit::iter() {
                                ui.selectable_value(
                                    &mut edited_prefs.temperature_unit,
                                    unit,
                                    unit.to_string(),
                                );
                            }
                        });
                    ui.end_row();

                    ui.label("Auto Open Camera");
                    ui.checkbox(&mut edited_prefs.auto_open_camera, "");
                    ui.end_row();

                    ui.label("Show unsupported cameras");
                    ui.checkbox(&mut edited_prefs.show_unsupported_cameras, "");
                    ui.end_row();
                });

            ui.add_space(10.0);
            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Save").clicked() {
                    global_state.prefs =
                        Some(self.local_user_preferences.as_ref().unwrap().clone());
                    self.local_user_preferences = None;
                    let _ = global_state
                        .prefs
                        .as_ref()
                        .context("Failed to get user preferences")
                        .map(|prefs| prefs.save())
                        .inspect_err(|err| error!("Failed to save user preferences: {}", err));
                }
                if ui.button("Cancel").clicked() {
                    self.local_user_preferences = None;
                }
            });
        }
    }

    fn force_close(&mut self) -> bool {
        self.local_user_preferences.is_none()
    }
}
