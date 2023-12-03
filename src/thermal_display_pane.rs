use std::{cell::RefCell, rc::Rc};

use eframe::{egui, epaint::Vec2};

use crate::{pane_dispatcher::Pane, AppGlobalState};

pub struct ThermalDisplayPane {
    global_state: Rc<RefCell<AppGlobalState>>,
    preview_zoom: f32,
    camera_texture: Option<egui::TextureHandle>,
}

impl ThermalDisplayPane {
    pub fn new(global_state: Rc<RefCell<AppGlobalState>>) -> ThermalDisplayPane {
        ThermalDisplayPane {
            global_state,
            preview_zoom: 1.0,
            camera_texture: None,
        }
    }
}

impl Pane for ThermalDisplayPane {
    fn title(&self) -> egui::WidgetText {
        "Thermal Display".into()
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        let scroll_delta = ui.ctx().input(|i| i.scroll_delta);
        if scroll_delta.y != 0.0 {
            self.preview_zoom += scroll_delta.y / 100.0;
            if self.preview_zoom < 0.1 {
                self.preview_zoom = 0.1;
            }
        }
        ui.centered_and_justified(|ui| {
            self.global_state
                .borrow()
                .last_thermal_capturer_result
                .as_ref()
                .and_then(|res| {
                    self.camera_texture = Some(ui.ctx().load_texture(
                        "cam_ctx",
                        res.image.clone(),
                        Default::default(),
                    ));
                    Some(())
                });
            self.camera_texture.as_ref().and_then(|texture| {
                ui.add(
                    egui::Image::new(texture)
                        .fit_to_fraction(Vec2::new(self.preview_zoom, self.preview_zoom)),
                );
                Some(())
            });
        });
    }
}
