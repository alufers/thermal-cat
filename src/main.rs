#![deny(elided_lifetimes_in_paths)]

use std::{cell::RefCell, collections::VecDeque, rc::Rc, time::SystemTime};

use chart_pane::ChartPane;
use dynamic_range_curve::DynamicRangeCurve;
use egui_dock::{DockArea, DockState, NodeIndex};
use gizmos::{Gizmo, GizmoKind};
use history_data_collector::HistoryDataCollector;
use hotplug_detector::{run_hotplug_detector, HotplugDetector};
use log::error;

use nokhwa::native_api_backend;

use eframe::{
    egui::{self},
    epaint::Color32,
    icon_data,
};
use pane_dispatcher::{Pane, PaneDispatcher};
use panes::{
    capture_pane::CapturePane,
    gallery_pane::{GalleryElement, GalleryPane},
    histogram_pane::HistogramPane,
    measurements_pane::MeasurementsPane,
    performance_stats_pane::PerformanceStatsPane,
    setup_pane::SetupPane,
    thermal_display_pane::ThermalDisplayPane,
    user_preferences_pane::UserPreferencesPane,
};
use recorders::recorder::RecorderState;
use temperature::{Temp, TempRange, TemperatureUnit};
use thermal_capturer::{ThermalCapturer, ThermalCapturerResult, ThermalCapturerSettings};
use types::image_rotation::ImageRotation;
use user_preferences::UserPreferences;
use video_thumbnail_loader::VideoThumbnailLoader;

mod auto_display_range_controller;
mod camera_adapter;
mod camera_enumerator;
mod chart_pane;
mod dynamic_range_curve;
mod gizmos;
mod gradient_selector_widget;
mod history_data_collector;
mod hotplug_detector;
mod pane_dispatcher;
mod panes;
mod recorders;
mod temperature;
mod temperature_edit_field;
mod thermal_capturer;
mod thermal_data;
mod thermal_gradient;
mod types;
mod user_preferences;
mod util;
mod video_thumbnail_loader;
mod widgets;

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 600.0])
            .with_icon(
                icon_data::from_png_bytes(&include_bytes!("../thermal-cat-logo-512px.png")[..])
                    .unwrap(),
            ),
        renderer: eframe::Renderer::Wgpu,

        ..Default::default()
    };
    eframe::run_native(
        "Thermal Cat",
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);

            cc.egui_ctx
                .add_image_loader(std::sync::Arc::new(VideoThumbnailLoader::default()));

            Ok(Box::<ThermalViewerApp>::default())
        }),
    )
}

pub struct AppGlobalState {
    did_try_open_camera_at_startup: bool,
    should_try_open_camera_on_next_hotplug: bool,

    thermal_capturer_inst: Option<ThermalCapturer>,
    thermal_capturer_settings: ThermalCapturerSettings,
    last_thermal_capturer_result: Option<Box<ThermalCapturerResult>>,

    hotplug_detector: Option<HotplugDetector>,
    history_data_collector: HistoryDataCollector,

    prefs: Option<UserPreferences>,

    // Thumbnails shown in the "Capture tab"
    gallery: VecDeque<GalleryElement>,
    did_init_gallery: bool,
}

impl AppGlobalState {
    fn preferred_temperature_unit(&self) -> TemperatureUnit {
        self.prefs
            .as_ref()
            .map(|p| p.temperature_unit)
            .unwrap_or_default()
    }
}

struct ThermalViewerApp {
    did_init: bool,

    dock_state: DockState<Box<dyn Pane>>,

    global_state: Rc<RefCell<AppGlobalState>>,
}

impl ThermalViewerApp {
    fn set_default_dock_state(&mut self) {
        self.dock_state = DockState::new(vec![Box::new(ThermalDisplayPane::new(
            self.global_state.clone(),
        ))]);
        let [right, left] = self.dock_state.main_surface_mut().split_left(
            NodeIndex::root(),
            0.3,
            vec![Box::new(SetupPane::new(self.global_state.clone()))],
        );

        self.dock_state.main_surface_mut().split_below(
            right,
            0.7,
            vec![
                Box::new(HistogramPane::new(self.global_state.clone())),
                Box::new(ChartPane::new(self.global_state.clone())),
                Box::new(GalleryPane::new(self.global_state.clone())),
            ],
        );

        self.dock_state.main_surface_mut().split_below(
            left,
            0.75,
            vec![Box::new(CapturePane::new(self.global_state.clone()))],
        );
        self.dock_state.main_surface_mut().split_below(
            left,
            0.8,
            vec![Box::new(MeasurementsPane::new(self.global_state.clone()))],
        );
    }
}

impl Default for ThermalViewerApp {
    fn default() -> Self {
        let _backend = native_api_backend().unwrap();
        let global_state = AppGlobalState {
            did_try_open_camera_at_startup: false,
            should_try_open_camera_on_next_hotplug: true,

            prefs: None,
            thermal_capturer_inst: None,
            thermal_capturer_settings: ThermalCapturerSettings {
                rotation: ImageRotation::None,
                auto_range: true,
                manual_range: TempRange::new(
                    Temp::from_unit(TemperatureUnit::Celsius, 0.0),
                    Temp::from_unit(TemperatureUnit::Celsius, 50.0),
                ),
                gradient: thermal_gradient::THERMAL_GRADIENTS[0].clone(),
                gizmo: Gizmo::new_root(vec![
                    Gizmo::new(GizmoKind::MaxTemp, "Max".to_string(), Color32::RED),
                    Gizmo::new(
                        GizmoKind::MinTemp,
                        "Min".to_string(),
                        Color32::from_rgb(72, 219, 251),
                    ),
                ]),
                dynamic_range_curve: DynamicRangeCurve::default(),
                recorders: vec![],
            },
            last_thermal_capturer_result: None,
            hotplug_detector: None,
            history_data_collector: HistoryDataCollector::new(),

            gallery: VecDeque::new(),
            did_init_gallery: false,
        };

        ThermalViewerApp {
            dock_state: DockState::new(vec![]),

            did_init: false,
            global_state: Rc::new(RefCell::new(global_state)),
        }
    }
}

impl eframe::App for ThermalViewerApp {
    fn update(&mut self, ctx: &egui::Context, _frame_egui: &mut eframe::Frame) {
        if !self.did_init {
            self.did_init = true;
            self.set_default_dock_state();
            let mut borrowed_global_state = self.global_state.borrow_mut();
            borrowed_global_state.prefs = Some(
                UserPreferences::load()
                    .inspect_err(|err| {
                        error!(
                            "Failed to load user preferences from {}: {}",
                            UserPreferences::preferences_path()
                                .to_string_lossy()
                                .to_string(),
                            err
                        )
                    })
                    .unwrap_or_default(),
            );
            let cloned_ctx = ctx.clone();

            borrowed_global_state.hotplug_detector = run_hotplug_detector(move |_| {
                cloned_ctx.request_repaint();
            })
            .inspect_err(|e| {
                error!("Failed to start hotplug detector: {}", e);
            })
            .ok();
            borrowed_global_state.should_try_open_camera_on_next_hotplug = borrowed_global_state
                .prefs
                .as_ref()
                .map(|p| p.auto_open_camera)
                .unwrap_or_default();
        }

        {
            let mut borrowed_global_state = self.global_state.borrow_mut();

            // drain thermal capturer results
            while {
                let mut had_result = false;
                if let Some(capturer) = borrowed_global_state.thermal_capturer_inst.as_mut() {
                    // Handle thermal capturer commands
                    if let Ok(r) = capturer.result_receiver.try_recv() {
                        match r {
                            Ok(result) => {
                                borrowed_global_state
                                    .history_data_collector
                                    .add_from_gizmo_results(
                                        result.capture_time,
                                        &result.gizmo_results,
                                    )
                                    .unwrap();

                                // Add captured image to gallery if needed
                                let mut gallery_tmp = vec![];
                                borrowed_global_state.thermal_capturer_settings.recorders =
                                    borrowed_global_state
                                        .thermal_capturer_settings
                                        .recorders
                                        .drain(..)
                                        .filter(|recorder| {
                                            let recorder = recorder.lock().unwrap();
                                            if recorder.state() == RecorderState::Done {
                                                for file in recorder.files_created() {
                                                    gallery_tmp.push(GalleryElement {
                                                        path: file,
                                                        created_at: SystemTime::now(),
                                                    });
                                                }
                                                return false;
                                            }
                                            true
                                        })
                                        .collect();
                                borrowed_global_state.gallery.extend(gallery_tmp);
                                borrowed_global_state.last_thermal_capturer_result = Some(result);

                                had_result = true;
                            }
                            Err(e) => {
                                error!("Thermal capturer error: {}", e);
                                borrowed_global_state.thermal_capturer_inst = None;
                            }
                        }
                    }
                }

                had_result
            } {}
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Preferences").clicked() {
                        // TODO: forbid opening multiple user preferences windows
                        self.dock_state
                            .add_window(vec![Box::new(UserPreferencesPane::new(
                                self.global_state.clone(),
                            ))]);
                    }
                    ui.separator();
                    if ui.button("Quit").clicked() {
                        self.global_state.borrow_mut().thermal_capturer_inst = None;
                        std::process::exit(0);
                    }
                });
                ui.menu_button("Window", |ui| {
                    if ui.button("Performance stats").clicked() {
                        self.dock_state
                            .add_window(vec![Box::new(PerformanceStatsPane::new(
                                self.global_state.clone(),
                            ))]);
                    }
                    if ui.button("Reset Layout").clicked() {
                        self.set_default_dock_state();
                    }
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // Either render a tab maximized, or render the egui_dock layout
            let fulscreen_tab = self
                .dock_state
                .iter_all_tabs_mut()
                .find(|tab| tab.1.is_maximized());

            if let Some(tab) = fulscreen_tab {
                tab.1.ui(ui);
            } else {
                DockArea::new(&mut self.dock_state)
                    .style(egui_dock::Style::from_egui(ui.style().as_ref()))
                    .show_inside(ui, &mut PaneDispatcher {});
            }
        });
    }
}
