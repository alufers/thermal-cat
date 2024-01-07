use std::{cell::RefCell, rc::Rc};

use eframe::{
    egui::{
        self,
        load::{TextureLoadResult, TexturePoll},
        DragValue, Image, Layout, Response, SizeHint, Slider, TextureOptions, Ui, Widget,
    },
    epaint::{TextureHandle, Vec2},
};
use egui_plot::{Plot, PlotBounds, PlotImage, PlotPoint};

use crate::{
    gizmos::GizmoKind, pane_dispatcher::Pane, thermal_data::ThermalDataPos, AppGlobalState,
};

pub struct ThermalDisplayPane {
    global_state: Rc<RefCell<AppGlobalState>>,

    camera_texture: Option<egui::TextureHandle>,
    camera_image_size: Option<(usize, usize)>,

    crosshair_texture_load_result: Option<TextureLoadResult>,
    crosshair_texture: Option<egui::TextureHandle>,

    zoom_factor: f64,
    center_offset: Vec2,
}

impl ThermalDisplayPane {
    pub fn new(global_state: Rc<RefCell<AppGlobalState>>) -> ThermalDisplayPane {
        ThermalDisplayPane {
            global_state,
            camera_texture: None,
            crosshair_texture_load_result: None,
            crosshair_texture: None,
            camera_image_size: None,

            zoom_factor: 1.0,
            center_offset: Vec2::new(0.0, 0.0),
        }
    }

    fn build_toolbar_ui(&mut self, ui: &mut egui::Ui) {
        ui.with_layout(
            Layout::left_to_right(egui::Align::Min).with_main_align(egui::Align::Min),
            |ui| {
                Image::new(egui::include_image!("./icons/zoom-in.svg"))
                    .max_height(16.0)
                    .max_width(16.0)
                    .tint(ui.style().visuals.widgets.active.fg_stroke.color)
                    .ui(ui);
                zoom_edit_field(ui, &mut self.zoom_factor);
                Slider::new(&mut self.zoom_factor, 0.1..=10.0)
                    .clamp_to_range(true)
                    .show_value(false)
                    .ui(ui);
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
            global_state
                .last_thermal_capturer_result
                .as_ref()
                .map(|res| {
                    self.camera_texture = Some(ui.ctx().load_texture(
                        "cam_ctx",
                        res.image.clone(),
                        TextureOptions {
                            magnification: egui::TextureFilter::Nearest,
                            ..Default::default()
                        },
                    ));
                    self.camera_image_size = Some((res.image.width(), res.image.height()));
                });

            let gizmo_results = global_state
                .last_thermal_capturer_result
                .as_ref()
                .map(|r| r.gizmo_results.clone())
                .clone();

            self.crosshair_texture_load_result.get_or_insert_with(|| {
                egui::include_image!("./icons/crosshair_center.svg").load(
                    ui.ctx(),
                    TextureOptions::default(),
                    SizeHint::Scale(5.0.into()),
                )
            });
            if self.crosshair_texture.is_none() {
                if let TexturePoll::Ready { texture } = self
                    .crosshair_texture_load_result
                    .as_ref()
                    .unwrap()
                    .as_ref()
                    .unwrap()
                {
                    self.crosshair_texture =
                        Some(TextureHandle::new(ui.ctx().tex_manager(), texture.id))
                }
            }
            ui.vertical(|ui| {
                self.build_toolbar_ui(ui);
                self.camera_texture.as_ref().map(|texture| {
                    let img_size = self.camera_image_size.unwrap();

                    Plot::new("thermal_display_plot")
                        .show_grid(false)
                        .show_axes(false)
                        .allow_boxed_zoom(false)
                        .allow_double_click_reset(false)
                        .allow_zoom(false)
                        .allow_drag(false)
                        .allow_scroll(false)
                        .data_aspect(1.0)
                        .show(ui, |plot_ui| {
                            // let's manually set the bounds we need

                            let center_x = img_size.0 as f64 / 2.0 + self.center_offset.x as f64;
                            let center_y = img_size.1 as f64 / 2.0 + self.center_offset.y as f64;

                            plot_ui.set_plot_bounds(PlotBounds::from_min_max(
                                [
                                    center_x - (img_size.0 as f64 / 2.0) * (1.0 / self.zoom_factor),
                                    center_y - (img_size.1 as f64 / 2.0) * (1.0 / self.zoom_factor),
                                ],
                                [
                                    center_x + (img_size.0 as f64 / 2.0) * (1.0 / self.zoom_factor),
                                    center_y + (img_size.1 as f64 / 2.0) * (1.0 / self.zoom_factor),
                                ],
                            ));

                            plot_ui.image(PlotImage::new(
                                texture,
                                PlotPoint::new(img_size.0 as f64 / 2.0, img_size.1 as f64 / 2.0),
                                Vec2::new(img_size.0 as f32, img_size.1 as f32),
                            ));

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

                                        let point = PlotPoint::new(x, y);
                                        let _size = 10.0;
                                        if let Some(crosshair) = self.crosshair_texture.as_ref() {
                                            // white backdrop for contrast
                                            plot_ui.image(PlotImage::new(
                                                crosshair,
                                                point,
                                                // 5 seems okay
                                                Vec2::new(6.0, 6.0),
                                            ));
                                            plot_ui.image(
                                                PlotImage::new(
                                                    crosshair,
                                                    point,
                                                    // 5 seems okay
                                                    Vec2::new(5.0, 5.0),
                                                )
                                                .tint(c.color),
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
                                    let zoom_delta_from_scroll = (inp.scroll_delta.y / 200.0).exp();

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

                                if zoom_delta != 0.0 {
                                    self.zoom_factor *= zoom_delta as f64;
                                    self.zoom_factor = self.zoom_factor.max(0.1);
                                    self.zoom_factor = self.zoom_factor.min(10.0);

                                    if (1.0 - self.zoom_factor).abs() < 0.055 {
                                        // snap to 1.0
                                        self.zoom_factor = 1.0;
                                    }
                                }
                            }
                            if plot_ui.response().dragged()
                                && plot_ui.response().ctx.input(|inp| {
                                    inp.pointer.button_down(egui::PointerButton::Middle)
                                })
                            {
                                let delta = plot_ui.response().drag_delta();

                                self.center_offset += delta
                                    * Vec2::new(
                                        (-1.0 / plot_ui.transform().dpos_dvalue_x()) as f32,
                                        (-1.0 / plot_ui.transform().dpos_dvalue_y()) as f32,
                                    );
                            }
                        });
                });
            });
        });
    }
}

pub fn zoom_edit_field(ui: &mut Ui, zoom_value: &mut f64) -> Response {
    let mut tmp_value = *zoom_value * 100.0;
    let res = ui.add(
        DragValue::new(&mut tmp_value)
            .speed(5.0)
            .max_decimals(0)
            .suffix("%")
            .clamp_range(10.0..=1000.0),
    );
    *zoom_value = tmp_value / 100.0;
    res
}
