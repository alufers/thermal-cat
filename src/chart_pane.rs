use std::{
    cell::RefCell,
    rc::Rc,
    time::{Duration, Instant},
};

use eframe::{egui, emath::Vec2b, epaint::Vec2};
use egui_plot::{Line, Plot, PlotPoints};

use crate::{pane_dispatcher::Pane, AppGlobalState};

pub struct ChartPane {
    global_state: Rc<RefCell<AppGlobalState>>,
    display_duration: Duration,
}

impl ChartPane {
    const POSSIBLE_DURATIONS: [Duration; 3] = [
        Duration::from_secs(60 * 15),
        Duration::from_secs(60 * 5),
        Duration::from_secs(60),
    ];
    pub fn new(global_state: Rc<RefCell<AppGlobalState>>) -> ChartPane {
        ChartPane {
            global_state,
            display_duration: Self::POSSIBLE_DURATIONS[2],
        }
    }

    fn duration_to_string(duration: Duration) -> String {
        let minutes = duration.as_secs() / 60;
        let seconds = duration.as_secs() % 60;
        let mut str = "".to_string();
        if minutes > 0 {
            str.push_str(&format!("{}m", minutes));
        }
        if seconds > 0 {
            str.push_str(&format!("{}s", seconds));
        }
        str
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
        let unit_suffix_clone = unit_suffix.clone(); // TODO: fixme
        egui::MenuBar::new().ui(ui, |ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                Self::POSSIBLE_DURATIONS.iter().for_each(|&duration| {
                    if ui
                        .selectable_value(
                            &mut self.display_duration,
                            duration,
                            Self::duration_to_string(duration),
                        )
                        .changed()
                    {}
                });
            });
        });

        let plot_ret = Plot::new("Chart")
            .auto_bounds(Vec2b::TRUE)
            .set_margin_fraction(Vec2::new(0.0, 0.1))
            .include_x(0.0)
            .include_x(-self.display_duration.as_secs_f64())
            .allow_scroll(false)
            .allow_zoom(false)
            .allow_drag(false)
            .allow_boxed_zoom(false)
            .allow_double_click_reset(false)
            .y_axis_label(format!(
                "Temperature ({})",
                global_state.preferred_temperature_unit().suffix()
            ))
            .y_axis_formatter(move |grid_mark, _range| {
                format!("{:.0} {}", grid_mark.value, unit_suffix)
            })
            .x_axis_formatter(move |grid_mark, _range| {
                let dur = Duration::from_secs_f64(grid_mark.value.abs());
                ChartPane::duration_to_string(dur)
            })
            .label_formatter(move |lbl: &str, p| {
                format!("{:.0} {} {}", p.y, unit_suffix_clone, lbl)
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
                    let start_of_range = now - self.display_duration;
                    let mut points = vec![];
                    global_state.history_data_collector.for_each_data_point(
                        gizmo.uuid,
                        start_of_range,
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
                    let line = Line::new(gizmo.name.clone(), PlotPoints::new(points))
                        .color(gizmo.color)
                        .name(gizmo.name.clone());
                    plot_ui.line(line);
                })
            });

        if plot_ret.response.hovered() {
            let scroll_delta_y = ui.input(|i: &egui::InputState| i.smooth_scroll_delta.y);
            if scroll_delta_y != 0.0 {
                let duration_secs = self.display_duration.as_secs() as f64;
                let new_duration_secs: f64 = duration_secs - (scroll_delta_y as f64 / 3.0);
                let new_duration_secs = new_duration_secs.max(5.0);
                self.display_duration = Duration::from_secs_f64(new_duration_secs);
            }
        }
    }
}
