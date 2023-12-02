use eframe::{
    egui::{self, Grid, Response, Ui, Widget},
    epaint::{TextureHandle, Vec2},
};

use crate::thermal_gradient::{self, ThermalGradient, THERMAL_GRADIENTS};

pub struct GradientSelectorView {
    preview_textures: Vec<TextureHandle>,
}

impl GradientSelectorView {
    pub fn new() -> Self {
        Self {
            preview_textures: vec![],
        }
    }

    pub fn draw(&mut self, ui: &mut Ui, selected_gradient: &mut ThermalGradient) -> Response {
        if self.preview_textures.len() != THERMAL_GRADIENTS.len() {
            self.preview_textures = THERMAL_GRADIENTS
                .iter()
                .map(|gradient| {
                    let gradient_image = gradient.create_demo_image(256, 32);
                    ui.ctx().load_texture(
                        format!("gradient_{}", gradient.name),
                        gradient_image,
                        Default::default(),
                    )
                })
                .collect();
        }

        let mut selected_gradient_idx = THERMAL_GRADIENTS
            .iter()
            .position(|g| g as *const ThermalGradient == selected_gradient as *const ThermalGradient)
            .unwrap_or(0);

        let prev_selected_gradient_idx = selected_gradient_idx;
        let mut resp = ui
            .vertical(|ui| {
                ui.label("Select gradient");
                Grid::new("gradient_grid")
                    .num_columns(2)
                    .spacing([10.0, 10.0])
                    .striped(true)
                    .max_col_width(200.0)
                    .show(ui, |ui| {
                        THERMAL_GRADIENTS
                            .iter()
                            .enumerate()
                            .for_each(|(i, gradient)| {
                                ui.radio_value(
                                    &mut selected_gradient_idx,
                                    i,
                                    gradient.name.clone(),
                                );
                                if ui
                                    .add(
                                        egui::Image::new(&self.preview_textures[i])
                                            .fit_to_fraction(Vec2::new(1.0, 1.0)),
                                    )
                                    .clicked()
                                {
                                    selected_gradient_idx = i;
                                }
                                ui.end_row();
                            });
                    });
            })
            .response;

        if prev_selected_gradient_idx != selected_gradient_idx {
            resp.mark_changed();
            println!(
                "Gradient changed to {}",
                THERMAL_GRADIENTS[selected_gradient_idx].name
            );
            *selected_gradient = THERMAL_GRADIENTS[selected_gradient_idx].clone();
        }

        resp
    }
}
