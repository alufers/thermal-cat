use std::{
    f32::consts::E,
    fmt::Error,
    sync::{Arc, Mutex},
};

use log::{debug, error, info, log_enabled, Level};

use camera_enumerator::{enumerate_cameras, EnumeratedCamera};
use gradient_selector_widget::GradientSelectorView;
use nokhwa::{native_api_backend, utils::CameraIndex, Camera};

use eframe::{
    egui::{self, Button, Id, Response},
    epaint::{text::LayoutJob, ColorImage, Vec2},
};
use thermal_capturer::{ThermalCapturer, ThermalCapturerResult};
use user_preferences::UserPreferences;
use user_preferences_window::UserPreferencesWindow;

use temperature_edit_field::temperature_edit_field;

mod camera_adapter;
mod camera_enumerator;
mod gradient_selector_widget;
mod temperature_edit_field;
mod temperature_unit;
mod thermal_capturer;
mod thermal_data;
mod thermal_gradient;
mod user_preferences;
mod user_preferences_window;

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([900.0, 600.0]),
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };
    eframe::run_native(
        "Open Desktop Thermal Viewer",
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);

            Box::<ThermalViewerApp>::default()
        }),
    )
}

struct ThermalViewerApp {
    did_init: bool,
    prefs: Option<UserPreferences>,

    user_preferences_window: UserPreferencesWindow,

    cameras: Vec<EnumeratedCamera>,
    selected_camera_index: CameraIndex,
    open_camera_error: Option<String>,

    thermal_capturer_inst: Option<ThermalCapturer>,

    preview_zoom: f32,
    camera_texture: Option<egui::TextureHandle>,
    incoming_image: Arc<Mutex<Option<ColorImage>>>,

    auto_range: bool,
    range_min: Arc<Mutex<f32>>,
    range_max: Arc<Mutex<f32>>,

    gradient_selector: GradientSelectorView,
}

impl ThermalViewerApp {
    fn selected_camera_info(&self) -> Option<&EnumeratedCamera> {
        self.cameras
            .iter()
            .find(|camera| camera.info.index() == &self.selected_camera_index)
    }

    fn open_selected_camera(&mut self, ctx: &egui::Context) {
        if let Some(adapter) = self.selected_camera_info().and_then(|i| i.adapter.as_ref()) {
            let cloned_ctx = ctx.clone();
            let cloned_incoming_image = self.incoming_image.clone();
            let cloned_adapter = adapter.clone();

            let cloned_range_min = self.range_min.clone();
            let cloned_range_max = self.range_max.clone();
            let _ = Camera::new(
                self.selected_camera_index.clone(),
                adapter.requested_format(),
            )
            .and_then(|cam| {
                // Create thermal capturer
                self.thermal_capturer_inst = Some(ThermalCapturer::new(
                    cam,
                    cloned_adapter,
                    Arc::new(move |result: ThermalCapturerResult| {
                        cloned_incoming_image.lock().unwrap().replace(result.image);
                        cloned_ctx.request_repaint();
                        
                        *cloned_range_min.lock().unwrap() = result.range_min;
                        *cloned_range_max.lock().unwrap() = result.range_max;
                      
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

impl Default for ThermalViewerApp {
    fn default() -> Self {
        let backend = native_api_backend().unwrap();
        let cameras = enumerate_cameras().unwrap();
        Self {
            did_init: false,
            prefs: None,
            user_preferences_window: UserPreferencesWindow::new(),

            selected_camera_index: cameras
                .iter()
                .find(|camera| camera.adapter.is_some())
                .map(|camera| camera.info.index().clone())
                .unwrap_or(CameraIndex::Index(0)),
            cameras,

            thermal_capturer_inst: None,
            camera_texture: None,
            open_camera_error: None,
            incoming_image: Arc::new(Mutex::new(None)),
            preview_zoom: 1.0,
            gradient_selector: GradientSelectorView::new(),
            auto_range: true,
            range_min: Arc::new(Mutex::new(273.15)),
            range_max: Arc::new(Mutex::new(273.15 + 60.0)),
        }
    }
}

impl eframe::App for ThermalViewerApp {
    fn update(&mut self, ctx: &egui::Context, frame_egui: &mut eframe::Frame) {
        if !self.did_init {
            self.did_init = true;
            self.prefs = Some(
                UserPreferences::load()
                    .inspect_err(|err| error!("Failed to load user preferences: {}", err))
                    .unwrap_or_default(),
            );
            if self.prefs.as_ref().unwrap().auto_open_camera {
                self.open_selected_camera(ctx);
            }
        }
        self.user_preferences_window
            .draw(&ctx, &mut self.prefs.as_mut().unwrap());

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Preferences").clicked() {
                        self.user_preferences_window
                            .show(&mut self.prefs.as_mut().unwrap());
                    }
                    ui.separator();
                    if ui.button("Quit").clicked() {
                        self.thermal_capturer_inst = None;
                        std::process::exit(0);
                    }
                });
            });
        });
        egui::SidePanel::new(egui::panel::Side::Left, Id::new("left_sidepanel")).show(ctx, |ui| {
            ui.heading("Open Thermal Viewer");
            ui.separator();
            ui.label("Select Camera:");
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
                                || self
                                    .prefs
                                    .as_ref()
                                    .map(|p| p.show_unsupported_cameras)
                                    .unwrap_or(false)
                        })
                        .for_each(|(i, camera)| {
                            ui.selectable_value(
                                &mut self.selected_camera_index,
                                camera.info.index().clone(),
                                camera.rich_text_name(false),
                            );
                        });
                });

            if self.thermal_capturer_inst.is_none() {
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
                    self.open_selected_camera(ctx);
                }
            } else {
                if ui.button("Close Camera").clicked() {
                    self.thermal_capturer_inst = None;
                }
            }

            if let Some(error) = &self.open_camera_error {
                ui.colored_label(egui::Color32::RED, error);
            }

            ui.separator();

            ui.checkbox(&mut self.auto_range, "Auto Range");
            egui::Grid::new("range_grid").show(ui, |ui| {
                ui.set_enabled(!self.auto_range);
                ui.label("Min");
                ui.label("Max");
                ui.end_row();
                temperature_edit_field(
                    ui,
                    self.prefs
                        .as_ref()
                        .map(|p| p.temperature_unit)
                        .unwrap_or_default(),
                    &mut self.range_min.lock().unwrap(),
                );

                temperature_edit_field(
                    ui,
                    self.prefs
                        .as_ref()
                        .map(|p| p.temperature_unit)
                        .unwrap_or_default(),
                        &mut self.range_max.lock().unwrap(),
                );
                ui.end_row();
            });

            ui.separator();

            if self.gradient_selector.draw(ui).changed() && self.thermal_capturer_inst.is_some() {
                self.thermal_capturer_inst
                    .as_mut()
                    .unwrap()
                    .set_gradient(self.gradient_selector.selected_gradient().clone());
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let scroll_delta = ctx.input(|i| i.scroll_delta);
            if scroll_delta.y != 0.0 {
                self.preview_zoom += scroll_delta.y / 100.0;
                if self.preview_zoom < 0.1 {
                    self.preview_zoom = 0.1;
                }
            }
            ui.centered_and_justified(|ui| {
                self.incoming_image
                    .lock()
                    .unwrap()
                    .as_mut()
                    .and_then(|image| {
                        self.camera_texture =
                            Some(ctx.load_texture("cam_ctx", image.clone(), Default::default()));
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
        });
    }
}
