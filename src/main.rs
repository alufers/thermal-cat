use std::{sync::{Arc, Mutex}, f32::consts::E};

use gradient_selector_widget::GradientSelectorView;
use nokhwa::{
    native_api_backend,
    pixel_format::RgbFormat,
    query,
    utils::{CameraIndex, CameraInfo, RequestedFormat, RequestedFormatType, CameraFormat, Resolution, FrameFormat},
    Camera,
};

use eframe::{egui::{self, Id, Response}, epaint::{ColorImage, Vec2}};
use thermal_capturer::{ThermalCapturer, ThermalCapturerResult};
use thermal_gradient::THERMAL_GRADIENT_DEFAULT;

mod thermal_capturer;
mod thermal_gradient;
mod gradient_selector_widget;


fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([900.0, 600.0]),
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
    cameras: Vec<CameraInfo>,
    selected_camera_index: CameraIndex,
    open_camera_error: Option<String>,


    thermal_capturer_inst: Option<ThermalCapturer>,

    
    preview_zoom: f32,
    camera_texture: Option<egui::TextureHandle>,
    incoming_image: Arc<Mutex<Option<ColorImage>>>,

    gradient_selector: GradientSelectorView,
 
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
            open_camera_error: None,
            incoming_image: Arc::new(Mutex::new(None)),
            preview_zoom: 1.0,

            gradient_selector: GradientSelectorView::new(),
        }
    }
}

impl eframe::App for ThermalViewerApp {
    fn update(&mut self, ctx: &egui::Context, frame_egui: &mut eframe::Frame) {
      
        egui::SidePanel::new(egui::panel::Side::Left, Id::new("left_sidepanel")).show(ctx, |ui| {
            ui.heading("Open Thermal Viewer");
            ui.separator();
            ui.label("Select Camera:");
            egui::ComboBox::from_label("")
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
                        RequestedFormatType::Closest(CameraFormat::new(Resolution::new(256, 192 * 2), FrameFormat::YUYV, 25))
                    );
                    let cloned_ctx = ctx.clone();
                    let mut cloned_incoming_image = self.incoming_image.clone();

                    let _ = Camera::new(self.selected_camera_index.clone(), requested).and_then(|mut cam| {
                        self.thermal_capturer_inst = Some(ThermalCapturer::new(cam, Arc::new(move |result: ThermalCapturerResult| {
                            cloned_incoming_image.lock().unwrap().replace(result.image);
                            cloned_ctx.request_repaint();
                        }))).and_then(|mut capturer| {
                            capturer.start();
                            Some(capturer)
                        });
                        self.open_camera_error = None;
                       Ok(())
                    }).or_else(|err| {
                        self.open_camera_error = Some(format!("Failed to open camera: {}", err));
                        Err(err)
                    });
                    
                   
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
            
            if self.gradient_selector.draw(ui).changed() && self.thermal_capturer_inst.is_some() {
               
                self.thermal_capturer_inst.as_mut().unwrap().set_gradient(self.gradient_selector.selected_gradient().clone());

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
