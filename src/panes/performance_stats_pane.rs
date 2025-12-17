use std::{cell::RefCell, rc::Rc, time::Instant};

use crate::{pane_dispatcher::Pane, AppGlobalState};
use eframe::egui::{self, Grid, Vec2b};
use egui_plot::{HLine, Line, Plot};
use once_cell::sync::Lazy;

const CHART_SAMPLES: usize = 200;

pub static EPOCH: Lazy<Instant> = Lazy::new(Instant::now);

pub struct PerformanceStatsPane {
    global_state: Rc<RefCell<AppGlobalState>>,
    fps_chart_data: Vec<[f64; 2]>,
}

impl PerformanceStatsPane {
    pub fn new(global_state: Rc<RefCell<AppGlobalState>>) -> Self {
        Self {
            global_state,
            fps_chart_data: vec![[0.0, 0.0]; 100],
        }
    }
}

impl Pane for PerformanceStatsPane {
    fn title(&self) -> egui::WidgetText {
        "Performance stats".into()
    }
    fn ui(&mut self, ui: &mut egui::Ui) {
        let global_state_clone = self.global_state.clone();
        let global_state = global_state_clone.as_ref().borrow_mut();

        let curr_time_sec = global_state
            .last_thermal_capturer_result
            .as_ref()
            .map(|r| r.capture_time.duration_since(*EPOCH).as_secs_f64())
            .unwrap_or_else(|| Instant::now().duration_since(*EPOCH).as_secs_f64());

        if let Some(last_thermal_capturer_result) = &global_state.last_thermal_capturer_result {
            self.fps_chart_data
                .push([curr_time_sec, last_thermal_capturer_result.real_fps as f64]);

            if self.fps_chart_data.len() > 4 * CHART_SAMPLES {
                // keep last 100 samples
                self.fps_chart_data =
                    self.fps_chart_data[self.fps_chart_data.len() - CHART_SAMPLES..].to_vec();
            }
        }

        Grid::new("my_grid")
            .num_columns(2)
            .spacing([40.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                ui.label("Reported FPS");
                ui.label(
                    global_state
                        .last_thermal_capturer_result
                        .as_ref()
                        .map(|r| format!("{:.1}", r.reported_fps))
                        .unwrap_or("-".to_string()),
                );
                ui.end_row();

                ui.label("Actual FPS");
                ui.label(
                    global_state
                        .last_thermal_capturer_result
                        .as_ref()
                        .map(|r| format!("{:.1}", r.real_fps))
                        .unwrap_or("-".to_string()),
                );
                ui.end_row();

                ui.label("Chart");

                let reported_fps = global_state
                    .last_thermal_capturer_result
                    .as_ref()
                    .map(|r| r.reported_fps)
                    .unwrap_or(10.0);

                Plot::new("fps plot")
                    .auto_bounds(Vec2b::new(false, true))
                    .include_x(0.0)
                    .include_x(-4)
                    .include_y(reported_fps)
                    .include_y(0.0)
                    .show_axes(Vec2b::new(false, true))
                    .show_x(false)
                    .allow_boxed_zoom(false)
                    .allow_drag(false)
                    .allow_double_click_reset(false)
                    .allow_scroll(false)
                    .allow_zoom(false)
                    .show_grid(true)
                    .show_background(false)
                    .show(ui, |ui| {
                        ui.hline(
                            HLine::new("Reported fps", reported_fps)
                                .color(egui::Color32::from_rgb(200, 200, 200))
                                .width(2.0),
                        );

                        let adjusted_data = self.fps_chart_data
                            [self.fps_chart_data.len().saturating_sub(CHART_SAMPLES)..]
                            .iter()
                            .map(|[x, y]| [x - curr_time_sec, *y])
                            .collect::<Vec<[f64; 2]>>();

                        let line = Line::new("FPS", adjusted_data)
                            .color(egui::Color32::from_rgb(0, 255, 0))
                            .name("FPS");
                        ui.line(line);
                    });
            });
    }
}
