use std::{cell::RefCell, rc::Rc};

use eframe::{
    egui::{
        self,
        Button, DragValue, Image, Layout, Response, RichText, Slider, TextureOptions, Ui,
        Widget,
    },
    emath::Align2,
    epaint::{Color32, Vec2},
};
use egui_plot::{MarkerShape, Plot, PlotBounds, PlotImage, PlotPoint, Points, Text};

use crate::{
    gizmos::GizmoKind, pane_dispatcher::Pane, thermal_data::ThermalDataPos,
    widgets::selectable_image_label::SelectableImageLabel, AppGlobalState,
};

pub struct ThermalDisplayPane {
    global_state: Rc<RefCell<AppGlobalState>>,

    camera_texture: Option<egui::TextureHandle>,
    camera_image_size: Option<(usize, usize)>,

    zoom_to_fit: bool,
    external_zoom_factor: f64,
    external_zoom_factor_changed: bool,

    maximized: bool,
}

impl ThermalDisplayPane {
    pub fn new(global_state: Rc<RefCell<AppGlobalState>>) -> ThermalDisplayPane {
        ThermalDisplayPane {
            global_state,
            camera_texture: None,

            camera_image_size: None,
            zoom_to_fit: true,
            external_zoom_factor: 1.0,
            external_zoom_factor_changed: false,
            maximized: false,
        }
    }

    fn build_toolbar_ui(&mut self, ui: &mut egui::Ui, global_state: &mut AppGlobalState) {
        ui.with_layout(
            Layout::left_to_right(egui::Align::Min)
                .with_main_align(egui::Align::Min)
                .with_main_justify(false),
            |ui| {
                Image::new(egui::include_image!("../icons/zoom-in.svg"))
                    .max_height(16.0)
                    .max_width(16.0)
                    .tint(ui.style().visuals.widgets.active.fg_stroke.color)
                    .ui(ui);

                if ui
                    .add_enabled(!self.zoom_to_fit, Button::new("Reset zoom"))
                    .on_hover_text("Reset zoom to fit the screen")
                    .clicked()
                {
                    self.zoom_to_fit = true;
                }
                if zoom_edit_field(ui, &mut self.external_zoom_factor).changed() {
                    self.external_zoom_factor_changed = true;
                    self.zoom_to_fit = false;
                }
                if Slider::new(&mut self.external_zoom_factor, 0.1..=10.0)
                    .clamp_to_range(true)
                    .show_value(false)
                    .ui(ui)
                    .changed()
                {
                    self.external_zoom_factor_changed = true;
                    self.zoom_to_fit = false;
                }

                ui.add_space(8.0);

                if ui
                    .add(SelectableImageLabel::new(
                        false,
                        Image::new(egui::include_image!("../icons/rotate-ccw.svg"))
                            .max_height(14.0)
                            .tint(ui.style().visuals.widgets.active.fg_stroke.color),
                    ))
                    .clicked()
                {
                    global_state.thermal_capturer_settings.rotation =
                        global_state.thermal_capturer_settings.rotation.next();
                    let settings_clone = global_state.thermal_capturer_settings.clone();
                    if let Some(thermal_capturer) = global_state.thermal_capturer_inst.as_mut() {
                        thermal_capturer.set_settings(settings_clone);
                    }
                }

                if ui
                    .add(SelectableImageLabel::new(
                        false,
                        Image::new(egui::include_image!("../icons/rotate-cw.svg"))
                            .max_height(14.0)
                            .tint(ui.style().visuals.widgets.active.fg_stroke.color),
                    ))
                    .clicked()
                {
                    global_state.thermal_capturer_settings.rotation =
                        global_state.thermal_capturer_settings.rotation.prev();
                    let settings_clone = global_state.thermal_capturer_settings.clone();
                    if let Some(thermal_capturer) = global_state.thermal_capturer_inst.as_mut() {
                        thermal_capturer.set_settings(settings_clone);
                    }
                }

                ui.with_layout(
                    Layout::right_to_left(egui::Align::Min).with_main_align(egui::Align::Max),
                    |ui| {
                        if ui
                            .add_enabled(
                                global_state.thermal_capturer_inst.is_some(),
                                SelectableImageLabel::new(
                                    self.maximized,
                                    Image::new(egui::include_image!("../icons/maximize.svg"))
                                        .max_height(14.0),
                                ),
                            )
                            .clicked()
                        {
                            self.maximized = !self.maximized;
                        }

                        if global_state.thermal_capturer_inst.is_none() {
                            self.maximized = false;
                        }
                    },
                );
            },
        );
    }
}

impl Pane for ThermalDisplayPane {
    fn title(&self) -> egui::WidgetText {
        "Thermal Display".into()
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        let global_state_clone = self.global_state.clone();
        let mut global_state = global_state_clone.as_ref().borrow_mut();

        ui.centered_and_justified(|ui| {
            if let Some(res) = global_state.last_thermal_capturer_result.as_ref() {
                self.camera_texture = Some(ui.ctx().load_texture(
                    "cam_ctx",
                    res.image.clone(),
                    TextureOptions {
                        magnification: egui::TextureFilter::Nearest,
                        ..Default::default()
                    },
                ));
                self.camera_image_size = Some((res.image.width(), res.image.height()));
            }

            let gizmo_results = global_state
                .last_thermal_capturer_result
                .as_ref()
                .map(|r| r.gizmo_results.clone())
                .clone();

            ui.vertical(|ui| {
                self.build_toolbar_ui(ui, &mut global_state);
                if let Some(texture) = self.camera_texture.as_ref() {
                    let img_size = self.camera_image_size.unwrap();

                    let plot_response = Plot::new("thermal_display_plot")
                        .show_grid(false)
                        .show_axes(false)
                        .allow_boxed_zoom(false)
                        .allow_double_click_reset(false)
                        .allow_zoom(false)
                        .allow_drag(false)
                        .allow_scroll(false)
                        .data_aspect(1.0)
                        .show(ui, |plot_ui| {
                            if self.zoom_to_fit {
                                // let's manually set the bounds we need

                                let center_x = img_size.0 as f64 / 2.0;
                                let center_y = img_size.1 as f64 / 2.0;

                                plot_ui.set_plot_bounds(PlotBounds::from_min_max(
                                    [
                                        center_x - (img_size.0 as f64 / 2.0) * (1.0),
                                        center_y - (img_size.1 as f64 / 2.0) * (1.0),
                                    ],
                                    [
                                        center_x + (img_size.0 as f64 / 2.0) * (1.0),
                                        center_y + (img_size.1 as f64 / 2.0) * (1.0),
                                    ],
                                ));
                            }

                            if self.external_zoom_factor_changed {
                                self.external_zoom_factor_changed = false;
                                let curr_zoom_factor = (img_size.0 as f64
                                    / plot_ui.plot_bounds().width())
                                .max(img_size.1 as f64 / plot_ui.plot_bounds().height());
                                let zoom_delta =
                                    (self.external_zoom_factor / curr_zoom_factor) as f32;

                                plot_ui.zoom_bounds(
                                    Vec2::new(zoom_delta, zoom_delta),
                                    plot_ui.plot_bounds().center(),
                                )
                            }

                            plot_ui.image(PlotImage::new(
                                texture,
                                PlotPoint::new(img_size.0 as f64 / 2.0, img_size.1 as f64 / 2.0),
                                Vec2::new(img_size.0 as f32, img_size.1 as f32),
                            ));

                            let temp_unit = global_state.preferred_temperature_unit();
                            global_state
                                .thermal_capturer_settings
                                .gizmo
                                .children_mut()
                                .unwrap()
                                .iter()
                                .for_each(|c| {
                                    let result =
                                        gizmo_results.as_ref().and_then(|r| r.get(&c.uuid));
                                    if let Some(result) = result {
                                        let _color = c.color;

                                        let x = result.pos.x as f64;

                                        let y = img_size.1 as f64 - result.pos.y as f64;

                                        let _size = 10.0;

                                        plot_ui.points(
                                            Points::new(vec![[x, y]])
                                                .shape(MarkerShape::Circle)
                                                .radius(12.0)
                                                .filled(true)
                                                .color(Color32::BLACK.gamma_multiply(0.3)),
                                        );
                                        plot_ui.points(
                                            Points::new(vec![[x, y]])
                                                .shape(MarkerShape::Circle)
                                                .radius(8.0)
                                                .filled(false)
                                                .color(Color32::WHITE),
                                        );
                                        plot_ui.points(
                                            Points::new(vec![[x, y]])
                                                .shape(MarkerShape::Plus)
                                                .radius(12.0)
                                                .color(c.color),
                                        );

                                        if c.show_temperature_label {
                                            plot_ui.text(
                                                Text::new(
                                                    PlotPoint::new(x + 4.0, y),
                                                    RichText::new(format!(
                                                        "{:.1} {}",
                                                        result.temperature.to_unit(temp_unit),
                                                        temp_unit.suffix()
                                                    ))
                                                    .size(16.0)
                                                    .background_color(
                                                        Color32::BLACK.gamma_multiply(0.5),
                                                    )
                                                    .color(Color32::WHITE),
                                                )
                                                .anchor(Align2::LEFT_CENTER),
                                            );
                                        }
                                    }
                                });

                            if plot_ui.response().clicked() {
                                let pos = plot_ui.pointer_coordinate().unwrap();
                                let x = pos.x as usize;
                                let y = pos.y as usize;
                                if x > 0 && y > 0 && x < img_size.0 && y < img_size.1 {
                                    global_state.thermal_capturer_settings.gizmo.push_child(
                                        GizmoKind::TempAt {
                                            pos: ThermalDataPos::new(x, img_size.1 - y),
                                        },
                                        "Custom".to_string(),
                                    );

                                    let settings_clone =
                                        global_state.thermal_capturer_settings.clone();
                                    if let Some(thermal_capturer) =
                                        global_state.thermal_capturer_inst.as_mut()
                                    {
                                        thermal_capturer.set_settings(settings_clone);
                                    }
                                }
                            }

                            if plot_ui.response().hovered() {
                                let zoom_delta = plot_ui.ctx().input(|inp| {
                                    // try to get zoom delta from 3 different sources
                                    let zoom_delta_from_multitouch =
                                        inp.multi_touch().map(|touch| touch.zoom_delta);
                                    let zoom_delta_from_scroll =
                                        (inp.smooth_scroll_delta.y / 200.0).exp();

                                    let zoom_delta_from_zoom = inp
                                        .raw
                                        .events
                                        .iter()
                                        .filter_map(|e| match e {
                                            egui::Event::Zoom(zoom) => Some(*zoom),
                                            _ => None,
                                        })
                                        .reduce(|a, b| a + b);
                                    None.or(zoom_delta_from_multitouch)
                                        .or(zoom_delta_from_zoom)
                                        .unwrap_or(zoom_delta_from_scroll)
                                });

                                if zoom_delta != 1.0 {
                                    self.zoom_to_fit = false;
                                    plot_ui.zoom_bounds_around_hovered(Vec2::new(
                                        zoom_delta, zoom_delta,
                                    ))
                                }
                            }
                            if plot_ui.response().dragged_by(egui::PointerButton::Middle) {
                                self.zoom_to_fit = false;
                                let delta = plot_ui.response().drag_delta();

                                let plot_transform = Vec2::new(
                                    (-1.0 / plot_ui.transform().dpos_dvalue_x()) as f32,
                                    (-1.0 / plot_ui.transform().dpos_dvalue_y()) as f32,
                                );
                                plot_ui.translate_bounds(plot_transform * delta);
                            }
                        });

                    self.external_zoom_factor = (img_size.0 as f64
                        / plot_response.transform.bounds().width())
                    .max(img_size.1 as f64 / plot_response.transform.bounds().height())
                }
            });
        });
    }

    fn is_maximized(&self) -> bool {
        self.maximized
    }
}

pub fn zoom_edit_field(ui: &mut Ui, zoom_value: &mut f64) -> Response {
    let mut tmp_value = *zoom_value * 100.0;
    let res = ui.add(
        DragValue::new(&mut tmp_value)
            .speed(3.0)
            .max_decimals(0)
            .suffix("%")
            .range(10.0..=1000.0),
    );
    *zoom_value = tmp_value / 100.0;
    res
}
