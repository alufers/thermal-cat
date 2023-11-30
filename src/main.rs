use nokhwa::{
    native_api_backend,
    pixel_format::RgbFormat,
    query,
    utils::{CameraIndex, CameraInfo, RequestedFormat, RequestedFormatType},
    Camera,
};

use eframe::egui::{self, Id};

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
    opened_camera: Option<Camera>,

    camera_texture: Option<egui::TextureHandle>,
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
            opened_camera: None,
            camera_texture: None,
        }
    }
}

impl eframe::App for ThermalViewerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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
            if self.opened_camera.is_none() {
                if ui.button("Open Camera").clicked() {
                    // todo: do something
                    let requested = RequestedFormat::new::<RgbFormat>(
                        RequestedFormatType::AbsoluteHighestResolution,
                    );
                    self.opened_camera =
                        Some(Camera::new(self.selected_camera_index.clone(), requested).unwrap());
                    self.opened_camera.as_mut().unwrap().open_stream().unwrap();
                }
            } else {
                if ui.button("Close Camera").clicked() {
                    self.opened_camera.as_mut().unwrap().stop_stream().unwrap();
                    self.opened_camera = None;
                }
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.opened_camera.is_some() {
                let frame = self.opened_camera.as_mut().unwrap().frame().unwrap();
                println!("Captured Single Frame of {}", frame.buffer().len());
                let decoded = frame.decode_image::<RgbFormat>().unwrap();
                let image = egui::ColorImage::from_rgb(
                    [decoded.width() as usize, decoded.height() as usize],
                    decoded.as_raw(),
                );
                self.camera_texture = Some(ui.ctx().load_texture(
                    "camera_ing",
                    image,
                    Default::default(),
                ));
                ui.centered_and_justified(|ui| {
                    ui.add(
                        egui::Image::new(self.camera_texture.as_ref().unwrap()).max_height(320.0),
                    );
                });

                ui.ctx().request_repaint();
            }
        });
    }
}
