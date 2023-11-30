use std::sync::{Arc, Mutex};

use nokhwa::{
    native_api_backend,
    pixel_format::RgbFormat,
    query,
    utils::{CameraIndex, CameraInfo, RequestedFormat, RequestedFormatType},
    Camera,
};

use eframe::{egui::{self, Id}, epaint::{ColorImage, Vec2}};
use thermal_capturer::ThermalCapturer;

mod thermal_capturer;
fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([900.0, 600.0]),
        ..Default::default()
    };
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);

            Box::<ThermalViewerApp>::default()
        }),
    )
}

struct ThermalViewerApp {
    cameras: Vec<CameraInfo>,
    selected_camera_index: CameraIndex,

    thermal_capturer_inst: Option<ThermalCapturer>,

    
    preview_zoom: f32,
    camera_texture: Option<egui::TextureHandle>,
    incoming_image: Arc<Mutex<Option<ColorImage>>>,
}

impl ThermalViewerApp {
    fn selected_camera_info(&self) -> Option<&CameraInfo> {
        self.cameras
            .iter()
            .find(|camera| camera.index() == &self.selected_camera_index)
    }
}

impl Default for ThermalViewerApp {
    fn default() -> Self {
        let backend = native_api_backend().unwrap();

        Self {
            cameras: query(backend).unwrap(),
            selected_camera_index: CameraIndex::Index(0),
            thermal_capturer_inst: None,
            camera_texture: None,
            incoming_image: Arc::new(Mutex::new(None)),
            preview_zoom: 1.0,
        }
    }
}

impl eframe::App for ThermalViewerApp {
    fn update(&mut self, ctx: &egui::Context, frame_egui: &mut eframe::Frame) {
        egui::SidePanel::new(egui::panel::Side::Left, Id::new("left_sidepanel")).show(ctx, |ui| {
            ui.heading("Open Thermal Viewer");

            egui::ComboBox::from_label("Camera")
                .selected_text(format!(
                    "#{} - {}",
                    self.selected_camera_index,
                    self.selected_camera_info()
                        .and_then(|camera| Some(camera.human_name()))
                        .unwrap_or("None".to_string())
                ))
                .width(200.0)
                .show_ui(ui, |ui| {
                    self.cameras.iter().enumerate().for_each(|(i, camera)| {
                        ui.selectable_value(
                            &mut self.selected_camera_index,
                            camera.index().clone(),
                            format!("#{} - {}", i, camera.human_name()),
                        );
                    });
                });
            if self.thermal_capturer_inst.is_none() {
                if ui.button("Open Camera").clicked() {
                    // todo: do something
                    let requested = RequestedFormat::new::<RgbFormat>(
                        RequestedFormatType::AbsoluteHighestResolution,
                    );
                    let cam = Camera::new(self.selected_camera_index.clone(), requested).unwrap();
                    let cloned_ctx = ctx.clone();
                    let mut cloned_incoming_image = self.incoming_image.clone();
                    self.thermal_capturer_inst = Some(ThermalCapturer::new(cam, Arc::new(move |image: ColorImage| {
                        cloned_incoming_image.lock().unwrap().replace(image);
                        cloned_ctx.request_repaint();
                    }))).and_then(|mut capturer| {
                        capturer.start();
                        Some(capturer)
                    });
                }
            } else {
                if ui.button("Close Camera").clicked() {
                    self.thermal_capturer_inst = None;
                }
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let scroll_delta =  ctx.input(|i| i.scroll_delta);
            if scroll_delta.y != 0.0 {
                self.preview_zoom += scroll_delta.y / 100.0;
                if self.preview_zoom < 0.1 {
                    self.preview_zoom = 0.1;
                }
            }
            ui.centered_and_justified(|ui| {
                self.incoming_image.lock().unwrap().as_mut().and_then(|image| {
                    self.camera_texture = Some(
                        ctx.load_texture("cam_ctx", image.clone(), Default::default())
                    );
                    Some(())
                });
                self.camera_texture.as_ref().and_then(|texture| {
                    ui.add(egui::Image::new(texture).fit_to_fraction(Vec2::new(self.preview_zoom, self.preview_zoom)));
                    Some(())
                });
            });
        });
    }
}
