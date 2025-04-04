use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use anyhow::Error;
use eframe::egui::{self, Button, CollapsingHeader};
use eframe::egui::{RichText, WidgetText};
use eframe::epaint::text::LayoutJob;
use nokhwa::utils::CameraIndex;
use nokhwa::Camera;

use crate::camera_enumerator::{enumerate_cameras, EnumeratedCamera};
use crate::dynamic_range_curve::dynamic_curve_editor;
use crate::gradient_selector_widget::GradientSelectorView;
use crate::pane_dispatcher::Pane;

use crate::temperature_edit_field::temperature_range_edit_field;
use crate::thermal_capturer::ThermalCapturer;
use crate::types::image_rotation::ImageRotation;
use crate::AppGlobalState;

use anyhow::{Context, Result};

pub struct SetupPane {
    global_state: Rc<RefCell<AppGlobalState>>,
    cameras: Result<Vec<EnumeratedCamera>, Error>,
    selected_camera_index: CameraIndex,
    open_camera_error: Option<String>,
    gradient_selector: GradientSelectorView,
}

impl SetupPane {
    pub fn new(global_state: Rc<RefCell<AppGlobalState>>) -> SetupPane {
        let cameras = enumerate_cameras().inspect_err(|err| {
            eprintln!("Failed to enumerate cameras: {:#}", err);
        });

        SetupPane {
            global_state,
            selected_camera_index: cameras
                .as_ref()
                .ok()
                .and_then(|cameras| {
                    cameras
                        .iter()
                        .find(|camera| camera.adapter.is_some())
                        .map(|camera| camera.info.index().clone())
                })
                .unwrap_or(CameraIndex::Index(0)),
            cameras,
            open_camera_error: None,
            gradient_selector: GradientSelectorView::new(),
        }
    }

    fn selected_camera_info(&self) -> Option<&EnumeratedCamera> {
        self.cameras.as_ref().ok().and_then(|cameras| {
            cameras
                .iter()
                .find(|camera| camera.info.index() == &self.selected_camera_index)
        })
    }

    fn open_selected_camera(
        &mut self,
        ctx: &egui::Context,
        global_state: &mut AppGlobalState,
    ) -> Result<()> {
        let adapter = self
            .selected_camera_info()
            .and_then(|i| i.adapter.as_ref())
            .context("No camera selected")?;
        
        let cloned_ctx = ctx.clone();
        let cloned_adapter = adapter.clone();

        Camera::new(
            self.selected_camera_index.clone(),
            adapter.requested_format(),
        )
        .map(|cam| {
            // Create thermal capturer

            global_state.thermal_capturer_inst = Some(ThermalCapturer::new(
                cam,
                cloned_adapter,
                global_state.thermal_capturer_settings.clone(),
                Arc::new(move || {
                    cloned_ctx.request_repaint(); // repaint so that the result can be read out
                }),
            ))
            .map(|mut capturer| {
                capturer.start();
                capturer
            });
            self.open_camera_error = None;
        })
        .inspect_err(|err| {
            self.open_camera_error = Some(format!("Failed to open camera: {}", err));
        })
        .context("Failed to open camera")
    }
}

impl Pane for SetupPane {
    fn title(&self) -> WidgetText {
        "Setup".into()
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        let global_state_clone = self.global_state.clone();
        let mut global_state = global_state_clone.as_ref().borrow_mut();
        if !global_state.did_try_open_camera_at_startup {
            global_state.did_try_open_camera_at_startup = true;
            if global_state.prefs.as_ref().unwrap().auto_open_camera {
                let _ = self.open_selected_camera(ui.ctx(), &mut global_state);
            }
        }

        if let Some(_evt) = global_state
            .hotplug_detector
            .as_mut()
            .and_then(|r| r.receiver.try_recv().ok())
        {
            self.cameras = enumerate_cameras().inspect_err(|err| {
                eprintln!("Failed to enumerate cameras: {:#}", err);
            });
            if global_state.should_try_open_camera_on_next_hotplug
                && global_state.thermal_capturer_inst.is_none()
            {
                // select a camera with an adapter if possible
                if !self
                    .selected_camera_info()
                    .as_ref()
                    .map(|i| i.adapter.is_some())
                    .unwrap_or(false)
                {
                    self.selected_camera_index = self
                        .cameras
                        .as_ref()
                        .ok()
                        .and_then(|cameras| {
                            cameras
                                .iter()
                                .find(|camera| camera.adapter.is_some())
                                .map(|camera| camera.info.index().clone())
                        })
                        .unwrap_or(CameraIndex::Index(0));
                }

                // try to open the camera
                let _ = self.open_selected_camera(ui.ctx(), &mut global_state);
            }
        }

        ui.heading("Open Thermal Viewer");
        ui.separator();
        ui.label("Select Camera");

        match self.cameras {
            Ok(ref cameras) => {
                egui::ComboBox::from_label("")
                    .selected_text(
                        self.selected_camera_info()
                            .map(|c| c.rich_text_name(true))
                            .unwrap_or(LayoutJob::single_section(
                                "No Camera Selected".to_string(),
                                Default::default(),
                            )),
                    )
                    .width(200.0)
                    .show_ui(ui, |ui| {
                        cameras
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
            }
            Err(ref err) => {
                ui.colored_label(
                    egui::Color32::RED,
                    format!("Camera enumeration error: {}", err),
                );
            }
        }

        if global_state.should_try_open_camera_on_next_hotplug
            && global_state.thermal_capturer_inst.is_none()
        {
            ui.colored_label(
                egui::Color32::GREEN,
                "Plug in a supported camera to start preview.",
            );
        }

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
                let _ = self.open_selected_camera(ui.ctx(), &mut global_state);
                global_state.should_try_open_camera_on_next_hotplug = true;
            }
        } else if ui.button("Close Camera").clicked() {
            global_state.thermal_capturer_inst = None;
            global_state.should_try_open_camera_on_next_hotplug = false;
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
                    ImageRotation::None,
                    "None",
                )
                .changed()
                || ui
                    .selectable_value(
                        &mut global_state.thermal_capturer_settings.rotation,
                        ImageRotation::Clockwise90,
                        "90°",
                    )
                    .changed()
                || ui
                    .selectable_value(
                        &mut global_state.thermal_capturer_settings.rotation,
                        ImageRotation::Clockwise180,
                        "180°",
                    )
                    .changed()
                || ui
                    .selectable_value(
                        &mut global_state.thermal_capturer_settings.rotation,
                        ImageRotation::Clockwise270,
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
        let mut range_copy = global_state
            .last_thermal_capturer_result
            .as_ref()
            .map(|r| r.image_range);
        if temperature_range_edit_field(
            ui,
            "range",
            !global_state.thermal_capturer_settings.auto_range,
            global_state.preferred_temperature_unit(),
            range_copy
                .as_mut()
                .and_then(|r| {
                    if global_state.thermal_capturer_settings.auto_range {
                        Some(r)
                    } else {
                        None
                    }
                })
                .unwrap_or(&mut global_state.thermal_capturer_settings.manual_range),
        )
        .changed()
        {
            let settings_clone = global_state.thermal_capturer_settings.clone();
            if let Some(thermal_capturer) = global_state.thermal_capturer_inst.as_mut() {
                thermal_capturer.set_settings(settings_clone);
            }
        }

        ui.separator();

        // Curve editor

        let has_modified_curve = !global_state
            .thermal_capturer_settings
            .dynamic_range_curve
            .is_default();

        let curve_heading = if has_modified_curve {
            RichText::new("Dynamic Range Curve *").strong()
        } else {
            RichText::new("Dynamic Range Curve")
        };

        CollapsingHeader::new(curve_heading)
            .id_source("curve_editor_header")
            .show(ui, |ui| {
                let manual_range = global_state.thermal_capturer_settings.manual_range;
                let curr_range = global_state
                    .last_thermal_capturer_result
                    .as_ref()
                    .map(|r| r.image_range)
                    .unwrap_or(manual_range);
                let unit = global_state.preferred_temperature_unit();
                if dynamic_curve_editor(
                    ui,
                    "main_curve_editor",
                    &mut global_state.thermal_capturer_settings,
                    curr_range,
                    unit,
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
