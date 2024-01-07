use std::{
    collections::HashMap,
    sync::{mpsc, Arc},
    thread,
};

use anyhow::{anyhow, Error};
use eframe::epaint::ColorImage;
use nokhwa::Camera;
use uuid::Uuid;

use crate::{
    auto_display_range_controller::AutoDisplayRangeController,
    camera_adapter::CameraAdapter,
    gizmos::{Gizmo, GizmoKind, GizmoResult},
    temperature::TempRange,
    thermal_data::{ThermalDataHistogram, ThermalDataRotation},
    thermal_gradient::ThermalGradient,
};

pub struct ThermalCapturerResult {
    pub image: ColorImage,
    pub image_range: TempRange,
    pub real_fps: f32,
    pub reported_fps: f32,
    pub histogram: ThermalDataHistogram,
    pub gizmo_results: HashMap<Uuid, GizmoResult>,
    pub capture_time: std::time::Instant,
}

#[derive(Clone)]
pub struct ThermalCapturerSettings {
    pub auto_range: bool,
    pub manual_range: TempRange,
    pub gradient: ThermalGradient,
    pub rotation: ThermalDataRotation,
    pub gizmo: Gizmo,
}

pub type ThermalCapturerCallback = Arc<dyn Fn() + Send + Sync>;

enum ThermalCapturerCmd {
    SetSettings(ThermalCapturerSettings),
    Stop,
}

struct ThermalCapturerCtx {
    camera: Camera,
    callback: ThermalCapturerCallback,
    cmd_receiver: mpsc::Receiver<ThermalCapturerCmd>,
    result_sender: mpsc::Sender<Result<Box<ThermalCapturerResult>, Error>>,
    adapter: Arc<dyn CameraAdapter>,
    settings: ThermalCapturerSettings,
    auto_range_controller: AutoDisplayRangeController,
    last_frame_time: std::time::Instant,
}

pub struct ThermalCapturer {
    ctx: Option<ThermalCapturerCtx>,
    cmd_sender: mpsc::Sender<ThermalCapturerCmd>,

    pub result_receiver: mpsc::Receiver<Result<Box<ThermalCapturerResult>, Error>>,
}

///
/// ThermalCapturer runs in a background thread continuously capturing images from the camera,
/// And calling the callback function with the captured image.
impl ThermalCapturer {
    pub fn new(
        camera: Camera,
        adapter: Arc<dyn CameraAdapter>,
        default_settings: ThermalCapturerSettings,
        callback: ThermalCapturerCallback,
    ) -> Self {
        let (cmd_sender, cmd_receiver) = mpsc::channel();
        let (result_sender, result_receiver) = mpsc::channel();
        Self {
            ctx: Some(ThermalCapturerCtx {
                camera,
                adapter,
                callback,
                cmd_receiver,
                result_sender,
                settings: default_settings,
                auto_range_controller: AutoDisplayRangeController::new(),
                last_frame_time: std::time::Instant::now(),
            }),
            cmd_sender,
            result_receiver,
        }
    }

    //
    pub fn start(&mut self) {
        // move the camera out of self so we can use it into the thread
        let mut ctx = self.ctx.take().unwrap();
        thread::spawn(move || {
            ctx.camera.open_stream().unwrap();

            fn produce_result(
                ctx: &mut ThermalCapturerCtx,
            ) -> Result<Box<ThermalCapturerResult>, Error> {
                ctx.last_frame_time = std::time::Instant::now();

                let thermal_data = ctx
                    .adapter
                    .capture_thermal_data(&mut ctx.camera)?
                    .rotated(ctx.settings.rotation);
                let capture_time = std::time::Instant::now();

                let (mintemp_pos, maxtemp_pos) = thermal_data.get_min_max_pos();

                let captured_range = TempRange::new(
                    thermal_data.temperature_at(mintemp_pos.x, mintemp_pos.y),
                    thermal_data.temperature_at(maxtemp_pos.x, maxtemp_pos.y),
                );

                let mut mapping_range = ctx.auto_range_controller.compute(captured_range);

                if !ctx.settings.auto_range {
                    mapping_range = ctx.settings.manual_range;
                }

                let image = thermal_data.map_to_image(|temp| {
                    ctx.settings.gradient.get_color(mapping_range.factor(temp))
                });

                let mut gizmo_results = HashMap::default();
                ctx.settings
                    .gizmo
                    .children_mut()
                    .ok_or(anyhow!("Root gizmo has no children"))?
                    .iter()
                    .for_each(|g| match g.kind {
                        GizmoKind::MaxTemp => {
                            gizmo_results.insert(
                                g.uuid,
                                GizmoResult {
                                    uuid: g.uuid,
                                    temperature: captured_range.max,
                                    pos: maxtemp_pos,
                                },
                            );
                        }
                        GizmoKind::MinTemp => {
                            gizmo_results.insert(
                                g.uuid,
                                GizmoResult {
                                    uuid: g.uuid,
                                    temperature: captured_range.min,
                                    pos: mintemp_pos,
                                },
                            );
                        }
                        GizmoKind::TempAt { pos } => {
                            gizmo_results.insert(
                                g.uuid,
                                GizmoResult {
                                    uuid: g.uuid,
                                    temperature: thermal_data.temperature_at(pos.x, pos.y),
                                    pos,
                                },
                            );
                        }
                        _ => panic!("Unimplemented gizmo kind"),
                    });

                Ok(Box::new(ThermalCapturerResult {
                    image,
                    real_fps: 1.0 / ctx.last_frame_time.elapsed().as_secs_f32(),
                    reported_fps: ctx.camera.frame_rate() as f32,
                    image_range: mapping_range,
                    histogram: ThermalDataHistogram::from_thermal_data(
                        &thermal_data,
                        captured_range.join(mapping_range),
                        100,
                    ),
                    gizmo_results,
                    capture_time,
                }))
            }
            loop {
                let result = produce_result(&mut ctx);
                ctx.result_sender.send(result).unwrap();
                (ctx.callback)();
                match ctx.cmd_receiver.try_recv() {
                    Ok(cmd) => match cmd {
                        ThermalCapturerCmd::Stop => {
                            ctx.camera.stop_stream().unwrap();
                            break;
                        }
                        ThermalCapturerCmd::SetSettings(range_settings) => {
                            ctx.settings = range_settings;
                        }
                    },
                    Err(_) => {}
                }
            }
        });
    }
    pub fn set_settings(&mut self, settings: ThermalCapturerSettings) {
        self.cmd_sender
            .send(ThermalCapturerCmd::SetSettings(settings))
            .unwrap();
    }
}

impl Drop for ThermalCapturer {
    fn drop(&mut self) {
        self.cmd_sender.send(ThermalCapturerCmd::Stop).unwrap();
    }
}
