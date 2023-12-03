use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use eframe::egui::WidgetText;
use eframe::egui::{self, Button};
use eframe::epaint::text::LayoutJob;
use nokhwa::utils::CameraIndex;
use nokhwa::Camera;

use crate::camera_enumerator::{enumerate_cameras, EnumeratedCamera};
use crate::gradient_selector_widget::GradientSelectorView;
use crate::pane_dispatcher::Pane;

use crate::temperature_edit_field::temperature_range_edit_field;
use crate::thermal_capturer::ThermalCapturer;
use crate::thermal_data::ThermalDataRotation;
use crate::AppGlobalState;

pub struct SetupPane {
    global_state: Rc<RefCell<AppGlobalState>>,

    did_init: bool,
    cameras: Vec<EnumeratedCamera>,
    selected_camera_index: CameraIndex,
    open_camera_error: Option<String>,
    gradient_selector: GradientSelectorView,
}

impl SetupPane {
    pub fn new(global_state: Rc<RefCell<AppGlobalState>>) -> SetupPane {
        let cameras = enumerate_cameras().unwrap();
        SetupPane {
            global_state,
            did_init: false,
            selected_camera_index: cameras
                .iter()
                .find(|camera| camera.adapter.is_some())
                .map(|camera| camera.info.index().clone())
                .unwrap_or(CameraIndex::Index(0)),
            cameras,
            open_camera_error: None,
            gradient_selector: GradientSelectorView::new(),
        }
    }

    fn selected_camera_info(&self) -> Option<&EnumeratedCamera> {
        self.cameras
            .iter()
            .find(|camera| camera.info.index() == &self.selected_camera_index)
    }

    fn open_selected_camera(&mut self, ctx: &egui::Context, global_state: &mut AppGlobalState) {
        if let Some(adapter) = self.selected_camera_info().and_then(|i| i.adapter.as_ref()) {
            let cloned_ctx = ctx.clone();
            let cloned_adapter = adapter.clone();

            let _ = Camera::new(
                self.selected_camera_index.clone(),
                adapter.requested_format(),
            )
            .and_then(|cam| {
                // Create thermal capturer

                global_state.thermal_capturer_inst = Some(ThermalCapturer::new(
                    cam,
                    cloned_adapter,
                    Arc::new(move || {
                        cloned_ctx.request_repaint(); // repaint so that the result can be read out
                    }),
                ))
                .and_then(|mut capturer| {
                    capturer.start();
                    Some(capturer)
                });
                self.open_camera_error = None;
                Ok(())
            })
            .or_else(|err| {
                self.open_camera_error = Some(format!("Failed to open camera: {}", err));
                Err(err)
            });
        }
    }
}

impl Pane for SetupPane {
    fn title(&self) -> WidgetText {
        "Setup".into()
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        let global_state_clone = self.global_state.clone();
        let mut global_state = global_state_clone.as_ref().borrow_mut();
        if !self.did_init {
            self.did_init = true;
            if global_state.prefs.as_ref().unwrap().auto_open_camera {
                self.open_selected_camera(ui.ctx(), &mut global_state);
            }
        }

        ui.heading("Open Thermal Viewer");
        ui.separator();
        ui.label("Select Camera");
        egui::ComboBox::from_label("")
            .selected_text(
                self.selected_camera_info()
                    .and_then(|c| Some(c.rich_text_name(true)))
                    .or(Some(LayoutJob::single_section(
                        "No Camera Selected".to_string(),
                        Default::default(),
                    )))
                    .unwrap(),
            )
            .width(200.0)
            .show_ui(ui, |ui| {
                self.cameras
                    .iter()
                    .enumerate()
                    .filter(|(_, cam)| {
                        cam.adapter.is_some()
                            || global_state
                                .prefs
                                .as_ref()
                                .map(|p| p.show_unsupported_cameras)
                                .unwrap_or(false)
                    })
                    .for_each(|(_i, camera)| {
                        ui.selectable_value(
                            &mut self.selected_camera_index,
                            camera.info.index().clone(),
                            camera.rich_text_name(false),
                        );
                    });
            });

        if global_state.thermal_capturer_inst.is_none() {
            // Show the "Open Camera" button only if the selected camera exists and has an adapter
            if ui
                .add_enabled(
                    self.selected_camera_info()
                        .and_then(|i| i.adapter.as_ref())
                        .is_some(),
                    Button::new("Open Camera"),
                )
                .clicked()
            {
                self.open_selected_camera(ui.ctx(), &mut global_state);
            }
        } else {
            if ui.button("Close Camera").clicked() {
                global_state.thermal_capturer_inst = None;
            }
        }

        if let Some(error) = &self.open_camera_error {
            ui.colored_label(egui::Color32::RED, error);
        }
        ui.separator();
        ui.label("Rotation");
        ui.horizontal(|ui| {
            if ui
                .selectable_value(
                    &mut global_state.thermal_capturer_settings.rotation,
                    ThermalDataRotation::None,
                    "None",
                )
                .changed()
                || ui
                    .selectable_value(
                        &mut global_state.thermal_capturer_settings.rotation,
                        ThermalDataRotation::Clockwise90,
                        "90°",
                    )
                    .changed()
                || ui
                    .selectable_value(
                        &mut global_state.thermal_capturer_settings.rotation,
                        ThermalDataRotation::Clockwise180,
                        "180°",
                    )
                    .changed()
                || ui
                    .selectable_value(
                        &mut global_state.thermal_capturer_settings.rotation,
                        ThermalDataRotation::Clockwise270,
                        "270°",
                    )
                    .changed()
            {
                let settings_clone = global_state.thermal_capturer_settings.clone();
                if let Some(thermal_capturer) = global_state.thermal_capturer_inst.as_mut() {
                    thermal_capturer.set_settings(settings_clone);
                }
            }
        });
        ui.separator();

        if ui
            .checkbox(
                &mut global_state.thermal_capturer_settings.auto_range,
                "Auto Range",
            )
            .changed()
        {
            // auto range has been disabled, copy the current range to the manual range
            if !global_state.thermal_capturer_settings.auto_range {
                let range_to_copy = global_state
                    .last_thermal_capturer_result
                    .as_ref()
                    .map(|res| res.image_range);
                if let Some(range) = range_to_copy {
                    global_state.thermal_capturer_settings.manual_range = range;
                }
            }
            let settings_clone = global_state.thermal_capturer_settings.clone();
            if let Some(thermal_capturer) = global_state.thermal_capturer_inst.as_mut() {
                thermal_capturer.set_settings(settings_clone);
            }
        }
        // copy of the range to pass to the edit field
        // (it will not be modified if auto_range is enabled, because the field is disabled)
        let mut range_copy;
        if temperature_range_edit_field(
            ui,
            "range",
            !global_state.thermal_capturer_settings.auto_range,
            global_state.preferred_temperature_unit(),
            if let Some(result) = global_state.last_thermal_capturer_result.as_ref()
                && (global_state.thermal_capturer_settings.auto_range)
            {
                range_copy = result.image_range;
                &mut range_copy
            } else {
                &mut global_state.thermal_capturer_settings.manual_range
            },
        )
        .changed()
        {
            let settings_clone = global_state.thermal_capturer_settings.clone();
            if let Some(thermal_capturer) = global_state.thermal_capturer_inst.as_mut() {
                thermal_capturer.set_settings(settings_clone);
            }
        }

        ui.separator();

        if self
            .gradient_selector
            .draw(ui, &mut global_state.thermal_capturer_settings.gradient)
            .changed()
        {
            let settings_clone = global_state.thermal_capturer_settings.clone();
            if let Some(thermal_capturer) = global_state.thermal_capturer_inst.as_mut() {
                thermal_capturer.set_settings(settings_clone.clone());
            }
        }
    }
}
