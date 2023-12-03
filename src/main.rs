#![feature(let_chains)]
#![deny(elided_lifetimes_in_paths)]

use std::{cell::RefCell, rc::Rc, sync::Arc};

use egui_dock::{DockArea, DockState};
use log::error;

use camera_enumerator::{enumerate_cameras, EnumeratedCamera};
use gradient_selector_widget::GradientSelectorView;
use nokhwa::{native_api_backend, utils::CameraIndex, Camera};

use eframe::{
    egui::{self, Button, Id, Style},
    epaint::{text::LayoutJob, Vec2},
};
use pane_dispatcher::{Pane, PaneDispatcher};
use setup_pane::SetupPane;
use temperature::{Temp, TempRange, TemperatureUnit};
use thermal_capturer::{ThermalCapturer, ThermalCapturerResult, ThermalCapturerSettings};
use thermal_display_pane::ThermalDisplayPane;
use user_preferences::UserPreferences;
use user_preferences_window::UserPreferencesWindow;

use temperature_edit_field::temperature_range_edit_field;

mod auto_display_range_controller;
mod camera_adapter;
mod camera_enumerator;
mod gradient_selector_widget;
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

pub struct AppGlobalState {
    thermal_capturer_inst: Option<ThermalCapturer>,
    thermal_capturer_settings: ThermalCapturerSettings,
    prefs: Option<UserPreferences>,
    last_thermal_capturer_result: Option<Box<ThermalCapturerResult>>,
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

impl Default for ThermalViewerApp {
    fn default() -> Self {
        let _backend = native_api_backend().unwrap();
        let mut global_state = AppGlobalState {
            prefs: None,
            thermal_capturer_inst: None,
            thermal_capturer_settings: ThermalCapturerSettings {
                auto_range: true,
                manual_range: TempRange::new(
                    Temp::from_unit(TemperatureUnit::Celsius, 0.0),
                    Temp::from_unit(TemperatureUnit::Celsius, 50.0),
                ),
                gradient: thermal_gradient::THERMAL_GRADIENTS[0].clone(),
            },
            last_thermal_capturer_result: None,
        };

        let mut me: ThermalViewerApp = ThermalViewerApp {
            dock_state: DockState::new(vec![]),

            did_init: false,

            user_preferences_window: UserPreferencesWindow::new(),
            global_state: Rc::new(RefCell::new(global_state)),
        };

        me
    }
}

impl eframe::App for ThermalViewerApp {
    fn update(&mut self, ctx: &egui::Context, _frame_egui: &mut eframe::Frame) {
        if !self.did_init {
            self.did_init = true;
            self.global_state.borrow_mut().prefs = Some(
                UserPreferences::load()
                    .inspect_err(|err| error!("Failed to load user preferences: {}", err))
                    .unwrap_or_default(),
            );
            self.dock_state = DockState::new(vec![
                Box::new(SetupPane::new(self.global_state.clone())),
                Box::new(ThermalDisplayPane::new(self.global_state.clone())),
            ]);
        }

        {
            let mut borrowed_global_state = self.global_state.borrow_mut();

            let mut result = Option::<Box<ThermalCapturerResult>>::None;

            if let Some(capturer) = borrowed_global_state.thermal_capturer_inst.as_mut() {
                // Handle thermal capturer commands
                if let Ok(cmd) = capturer.result_receiver.try_recv() {
                    result = Some(cmd);
                }
            }

            if let Some(result) = result {
                borrowed_global_state.last_thermal_capturer_result = Some(result);
            }
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
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            DockArea::new(&mut self.dock_state)
                .style(egui_dock::Style::from_egui(ui.style().as_ref()))
                .show_inside(ui, &mut PaneDispatcher {});
        });
    }
}
