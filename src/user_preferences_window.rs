use eframe::egui::{self, Button, Grid, Ui, ViewportBuilder, ViewportClass, ViewportId};

use crate::user_preferences::UserPreferences;

pub struct UserPreferencesWindow {
    local_user_preferences: Option<UserPreferences>,
}

impl UserPreferencesWindow {
    pub fn new() -> Self {
        Self {
            local_user_preferences: None,
        }
    }
    pub fn show(&mut self, user_preferences: &mut UserPreferences) {
        self.local_user_preferences = Some(user_preferences.clone());
    }
    pub fn draw(&mut self, ctx: &egui::Context, user_preferences: &mut UserPreferences) {
        if self.local_user_preferences.is_none() {
            return;
        }
        egui::Window::new("User Preferences")
            .id(egui::Id::new("user_preferences_window"))
            .default_pos(egui::Pos2::new(100.0, 100.0))
            .show(ctx, |ui| {
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


                    });


                ui.add_space(10.0);
                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() {
                        *user_preferences = self.local_user_preferences.as_ref().unwrap().clone();
                        self.local_user_preferences = None;
                    }
                    if ui.button("Cancel").clicked() {
                        self.local_user_preferences = None;
                    }
                })
            });
    }
}
