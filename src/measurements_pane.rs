use std::{cell::RefCell, rc::Rc};

use eframe::{
    egui::{
        self,
        color_picker::{color_picker_color32, Alpha},
        Area, Button, Frame, Grid, Image, ImageButton, Key, Order, Response, TextEdit, Ui, Widget,
    },
    epaint::Color32,
};

use crate::{gizmos::GizmoKind, pane_dispatcher::Pane, AppGlobalState};

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
                        let icon = Image::new(match gizmo.kind {
                            GizmoKind::MaxTemp => egui::include_image!("./icons/flame.svg"),
                            GizmoKind::MinTemp => egui::include_image!("./icons/snowflake.svg"),
                            GizmoKind::TempAt { pos: _ } => {
                                egui::include_image!("./icons/crosshair_center.svg")
                            }
                            _ => egui::include_image!("./icons/flame.svg"),
                        });

                        color_icon_rgb(
                            ui,
                            ImageButton::new(icon.tint(gizmo.color)).frame(false),
                            &mut gizmo.color,
                            Alpha::Opaque,
                        );

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

                        ui.add(TextEdit::singleline(&mut gizmo.name));
                        ui.end_row();
                    })
            });
    }
}

pub fn color_icon_rgb(ui: &mut Ui, icon: impl Widget, rgb: &mut Color32, alpha: Alpha) -> Response {
    let popup_id = ui.auto_id_with("popup");
    let open = ui.memory(|mem| mem.is_popup_open(popup_id));
    let mut button_response = ui.add(icon);
    if ui.style().explanation_tooltips {
        button_response = button_response.on_hover_text("Click to edit color");
    }

    if button_response.clicked() {
        ui.memory_mut(|mem| mem.toggle_popup(popup_id));
        println!("clicked");
    }

    const COLOR_SLIDER_WIDTH: f32 = 210.0;
    if ui.memory(|mem| mem.is_popup_open(popup_id)) {
        let area_response = Area::new(popup_id)
            .order(Order::Foreground)
            .fixed_pos(button_response.rect.max)
            .constrain(true)
            .show(ui.ctx(), |ui| {
                ui.spacing_mut().slider_width = COLOR_SLIDER_WIDTH;
                Frame::popup(ui.style()).show(ui, |ui| {
                    if color_picker_color32(ui, rgb, alpha) {
                        button_response.mark_changed();
                    }
                });
            })
            .response;

        if !button_response.clicked()
            && (ui.input(|i| i.key_pressed(Key::Escape)) || area_response.clicked_elsewhere())
        {
            ui.memory_mut(|mem| mem.close_popup());
        }
    }

    button_response
}