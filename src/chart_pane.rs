use std::{
    cell::RefCell,
    rc::Rc,
    time::{Duration, Instant},
};

use eframe::egui;
use egui_plot::{Line, Plot, PlotPoints};

use crate::{pane_dispatcher::Pane, AppGlobalState};

pub struct ChartPane {
    global_state: Rc<RefCell<AppGlobalState>>,
}

impl ChartPane {
    pub fn new(global_state: Rc<RefCell<AppGlobalState>>) -> ChartPane {
        ChartPane { global_state }
    }
}

impl Pane for ChartPane {
    fn title(&self) -> egui::WidgetText {
        "Chart".into()
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        let global_state_clone = self.global_state.clone();
        let mut global_state = global_state_clone.as_ref().borrow_mut();

        let unit_suffix = global_state.preferred_temperature_unit().suffix();

        Plot::new("Chart")
            .auto_bounds_x()
            .auto_bounds_y()
            .y_axis_label(format!(
                "Temperature ({})",
                global_state.preferred_temperature_unit().suffix()
            ))
            .x_axis_label("Time")
            // .include_y(0.0)
            // .include_y(30.0)
            // .y_axis_formatter(|factor, _max_chars, _range| format!("{:.0}%", factor))
            .y_axis_formatter(move |temp_val, _max_chars, _range| {
                return format!("{:.0} {}", temp_val, unit_suffix);
            })
            .show(ui, |plot_ui| {
                let gizmos = global_state
                    .thermal_capturer_settings
                    .gizmo
                    .children_mut()
                    .unwrap()
                    .clone(); // todo: remove clone

                gizmos.iter().for_each(|gizmo| {
                    let now = global_state
                        .last_thermal_capturer_result
                        .as_ref()
                        .map(|cr| cr.capture_time)
                        .unwrap_or(Instant::now());
                    let minute_ago = now - Duration::from_secs(60);
                    let mut points = vec![];
                    global_state.history_data_collector.for_each_data_point(
                        gizmo.uuid,
                        minute_ago,
                        now,
                        |data_point| {
                            points.push([
                                -(now - data_point.time).as_secs_f64(),
                                data_point
                                    .temperature
                                    .to_unit(global_state.preferred_temperature_unit())
                                    as f64,
                            ]);
                        },
                    );
                    let line = Line::new(PlotPoints::new(points))
                        .color(gizmo.color)
                        .name(gizmo.name.clone());
                    plot_ui.line(line);
                })
            });
    }
}
