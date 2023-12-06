use std::{cell::RefCell, rc::Rc};

use eframe::{
    egui::{self, TextureOptions},
    epaint::{Vec2},
};
use egui_plot::{Plot, PlotImage, PlotPoint, Points};

use crate::{
    gizmos::{GizmoKind},
    pane_dispatcher::Pane,
    thermal_data::ThermalDataPos,
    AppGlobalState,
};

pub struct ThermalDisplayPane {
    global_state: Rc<RefCell<AppGlobalState>>,

    camera_texture: Option<egui::TextureHandle>,
    camera_image_size: Option<(usize, usize)>,
    crosshair_texture: Option<egui::TextureHandle>,
}

impl ThermalDisplayPane {
    pub fn new(global_state: Rc<RefCell<AppGlobalState>>) -> ThermalDisplayPane {
        ThermalDisplayPane {
            global_state,
            camera_texture: None,
            crosshair_texture: None,
            camera_image_size: None,
        }
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
                .and_then(|res| {
                    self.camera_texture = Some(ui.ctx().load_texture(
                        "cam_ctx",
                        res.image.clone(),
                        TextureOptions {
                            magnification: egui::TextureFilter::Nearest,
                            ..Default::default()
                        }
                    ));
                    self.camera_image_size = Some((res.image.width(), res.image.height()));
                    Some(())
                });

            let gizmo_results = global_state
                .last_thermal_capturer_result
                .as_ref()
                .map(|r| r.gizmo_results.clone())
                .clone();

            // let crosshair_texture = self.crosshair_texture.get_or_insert_with(|| {
            //     let img: Image<'_> = Image::new(egui::include_image!("./icons/crosshair_center.svg"));
            //     ui.ctx().try_load_image(uri, size_hint)
            //     ui.ctx()
            //         .load_texture("crosshair_ctx", img, Default::default())
            // });

            self.camera_texture.as_ref().and_then(|texture| {
                let img_size = self.camera_image_size.unwrap();
                Plot::new("thermal_display_plot")
                    .show_grid(false)
                    .show_axes(false)
                    .data_aspect(1.0)
                    .auto_bounds_x()
                    .auto_bounds_y()
                    .show(ui, |plot_ui| {
                        let _points = global_state
                            .thermal_capturer_settings
                            .gizmo
                            .children_mut()
                            .unwrap()
                            .iter()
                            .for_each(|c| {
                                let result = gizmo_results.as_ref().and_then(|r| r.get(&c.uuid));
                                if let Some(result) = result {
                                    let _color = c.color;

                                    let x = result.pos.x as f64;

                                    let y = img_size.1 as f64 - result.pos.y as f64;

                                    let _point = PlotPoint::new(x, y);
                                    let _size = 10.0;
                                    // plot_ui.image(
                                    //     PlotImage::new(
                                    //         crosshair_texture,
                                    //         point,
                                    //         Vec2::new(size, size),
                                    //     )
                                    //     .tint(color),
                                    // )

                                    plot_ui.points(
                                        Points::new(vec![[x, y].into()]).color(c.color).radius(5.0),
                                    );
                                }
                            });

                        plot_ui.image(PlotImage::new(
                            texture,
                            PlotPoint::new(img_size.0 as f64 / 2.0, img_size.1 as f64 / 2.0),
                            Vec2::new(img_size.0 as f32, img_size.1 as f32),
                        ));

                        if plot_ui.response().clicked() {
                            let pos = plot_ui.pointer_coordinate().unwrap();

                            if pos.x > 0.0
                                && pos.y > 0.0
                                && pos.x < img_size.0 as f64
                                && pos.y < img_size.1 as f64
                            {
                                global_state.thermal_capturer_settings.gizmo.push_child(
                                    GizmoKind::TempAt {
                                        pos: ThermalDataPos::new(
                                            pos.x as usize,
                                            img_size.1 - pos.y as usize,
                                        ),
                                    },
                                    "Custom".to_string(),
                                );

                                let settings_clone = global_state.thermal_capturer_settings.clone();
                                if let Some(thermal_capturer) =
                                    global_state.thermal_capturer_inst.as_mut()
                                {
                                    thermal_capturer.set_settings(settings_clone);
                                }
                            }
                        }
                    });
                Some(())
            });
        });
    }
}
