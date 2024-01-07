#![feature(let_chains)]
#![deny(elided_lifetimes_in_paths)]

use std::{cell::RefCell, rc::Rc};


use chart_pane::ChartPane;
use egui_dock::{DockArea, DockState, NodeIndex};
use gizmos::{Gizmo, GizmoKind};
use histogram_pane::HistogramPane;
use history_data_collector::HistoryDataCollector;
use hotplug_detector::{run_hotplug_detector, HotplugDetector};
use log::error;

use measurements_pane::MeasurementsPane;
use nokhwa::native_api_backend;

use eframe::{
    egui::{self},
    epaint::Color32,
    icon_data,
};
use pane_dispatcher::{Pane, PaneDispatcher};
use setup_pane::SetupPane;
use temperature::{Temp, TempRange, TemperatureUnit};
use thermal_capturer::{ThermalCapturer, ThermalCapturerResult, ThermalCapturerSettings};
use thermal_data::ThermalDataRotation;
use thermal_display_pane::ThermalDisplayPane;
use user_preferences::UserPreferences;
use user_preferences_window::UserPreferencesWindow;

mod auto_display_range_controller;
mod camera_adapter;
mod camera_enumerator;
mod chart_pane;
mod gizmos;
mod gradient_selector_widget;
mod histogram_pane;
mod history_data_collector;
mod hotplug_detector;
mod measurements_pane;
mod pane_dispatcher;
mod setup_pane;
mod temperature;
mod temperature_edit_field;
mod thermal_capturer;
mod thermal_data;
mod thermal_display_pane;
mod thermal_gradient;
mod user_preferences;
mod user_preferences_window;

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
        "Open Desktop Thermal Viewer",
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);

            Box::<ThermalViewerApp>::default()
        }),
    )
}

pub struct AppGlobalState {
    did_try_open_camera_at_startup: bool,
    should_try_open_camera_on_next_hotplug: bool,

    thermal_capturer_inst: Option<ThermalCapturer>,
    thermal_capturer_settings: ThermalCapturerSettings,
    prefs: Option<UserPreferences>,
    last_thermal_capturer_result: Option<Box<ThermalCapturerResult>>,
    hotplug_detector: Option<HotplugDetector>,
    history_data_collector: HistoryDataCollector,
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

    user_preferences_window: UserPreferencesWindow,
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
            ],
        );

        self.dock_state.main_surface_mut().split_below(
            left,
            0.7,
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
                rotation: ThermalDataRotation::None,
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
            },
            last_thermal_capturer_result: None,
            hotplug_detector: None,
            history_data_collector: HistoryDataCollector::new(),
        };

        ThermalViewerApp {
            dock_state: DockState::new(vec![]),

            did_init: false,

            user_preferences_window: UserPreferencesWindow::new(),
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
                    .inspect_err(|err| error!("Failed to load user preferences: {}", err))
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
                    match capturer.result_receiver.try_recv() {
                        Ok(r) => match r {
                            Ok(result) => {
                                borrowed_global_state
                                    .history_data_collector
                                    .add_from_gizmo_results(
                                        result.capture_time,
                                        &result.gizmo_results,
                                    )
                                    .unwrap();
                                borrowed_global_state.last_thermal_capturer_result = Some(result);
                                had_result = true;
                            }
                            Err(e) => {
                                error!("Thermal capturer error: {}", e);
                                borrowed_global_state.thermal_capturer_inst = None;
                            }
                        },
                        Err(_) => {}
                    }
                }
               
                had_result
            } {}
        }

        self.user_preferences_window.draw(
            &ctx,
            &mut self.global_state.borrow_mut().prefs.as_mut().unwrap(),
        );

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Preferences").clicked() {
                        self.user_preferences_window
                            .show(&mut self.global_state.borrow_mut().prefs.as_mut().unwrap());
                    }
                    ui.separator();
                    if ui.button("Quit").clicked() {
                        self.global_state.borrow_mut().thermal_capturer_inst = None;
                        std::process::exit(0);
                    }
                });
                ui.menu_button("Window", |ui| {
                    if ui.button("Reset Layout").clicked() {
                        self.set_default_dock_state();
                    }
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            DockArea::new(&mut self.dock_state)
                .style(egui_dock::Style::from_egui(ui.style().as_ref()))
                .show_inside(ui, &mut PaneDispatcher {});
        });
    }
}
