use std::{cell::RefCell, rc::Rc};

use eframe::{
    egui::{
        self,
        color_picker::{color_picker_color32, Alpha},
        Grid, Image, TextEdit,
    },
    epaint::Color32,
};

use crate::{
    gizmos::GizmoKind, pane_dispatcher::Pane, AppGlobalState,
};

pub struct MeasurementsPane {
    global_state: Rc<RefCell<AppGlobalState>>,
}

impl MeasurementsPane {
    pub fn new(global_state: Rc<RefCell<AppGlobalState>>) -> MeasurementsPane {
        MeasurementsPane { global_state }
    }
}

impl Pane for MeasurementsPane {
    fn title(&self) -> egui::WidgetText {
        "Measurements".into()
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        let global_state_clone = self.global_state.clone();
        let mut global_state = global_state_clone.as_ref().borrow_mut();

        Grid::new("measurements_pane_grid")
            .striped(true)
            .num_columns(3)
            .show(ui, |ui| {
                ui.label("");
                ui.label("Name");
                ui.label("Value");
                ui.end_row();

                let gizmo_results = global_state
                    .last_thermal_capturer_result
                    .as_ref()
                    .map(|r| r.gizmo_results.clone())
                    .clone();

                let temp_unit = global_state.preferred_temperature_unit();

                global_state
                    .thermal_capturer_settings
                    .gizmo
                    .children_mut()
                    .unwrap()
                    .iter_mut()
                    .for_each(|gizmo| {
                        // ui.color_edit_button_srgba(&mut gizmo.color);
                        let icon = Image::new(match gizmo.kind {
                            GizmoKind::MaxTemp => egui::include_image!("./icons/flame.svg"),
                            GizmoKind::MinTemp => egui::include_image!("./icons/snowflake.svg"),
                            _ => egui::include_image!("./icons/flame.svg"),
                        });
                        ui.add(icon.tint(gizmo.color));

                        // color_picker_color32(ui, &mut gizmo.color, Alpha::Opaque);
                        ui.add(TextEdit::singleline(&mut gizmo.name).desired_width(200.0));
                        ui.label(
                            gizmo_results
                                .as_ref()
                                .and_then(|gr| gr.get(&gizmo.uuid).clone())
                                .map(|r| {
                                    format!(
                                        "{:.1} {}",
                                        r.temperature.to_unit(temp_unit),
                                        temp_unit.suffix()
                                    )
                                })
                                .unwrap_or(" - ".to_string()),
                        );
                        ui.end_row();
                    })
            });
    }
}
