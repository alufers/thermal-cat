use std::{cell::RefCell, rc::Rc};

use eframe::egui;
use egui_plot::{Bar, BarChart, Plot};

use crate::{pane_dispatcher::Pane, temperature::TemperatureUnit, AppGlobalState};

pub struct HistogramPane {
    global_state: Rc<RefCell<AppGlobalState>>,
}

impl HistogramPane {
    pub fn new(global_state: Rc<RefCell<AppGlobalState>>) -> HistogramPane {
        HistogramPane { global_state }
    }
}

impl Pane for HistogramPane {
    fn title(&self) -> egui::WidgetText {
        "Histogram".into()
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        let global_state_clone = self.global_state.clone();
        let global_state = global_state_clone.as_ref().borrow_mut();

        let default_vec = vec![];
        let temperature_points = global_state
            .last_thermal_capturer_result
            .as_ref()
            .map(|r| &r.histogram.points)
            .unwrap_or(&default_vec);

        let color_range = global_state
            .last_thermal_capturer_result
            .as_ref()
            .map(|r| r.image_range)
            .unwrap_or_default();

        let mut bucket_width = 1.0;
        if temperature_points.len() > 1 {
            bucket_width = (temperature_points[1].temperature - temperature_points[0].temperature)
                .to_unit(TemperatureUnit::Kelvin) as f64;
        }

        let chart = BarChart::new(
            temperature_points
                .iter()
                .map(|p| {
                    Bar::new(
                        p.temperature
                            .to_unit(global_state.preferred_temperature_unit())
                            as f64,
                        p.factor as f64 * 100.0,
                    )
                    .width(bucket_width)
                    .fill(
                        global_state
                            .thermal_capturer_settings
                            .gradient
                            .get_color(color_range.factor(p.temperature)),
                    )
                })
                .collect(),
        );
        let unit_suffix = global_state.preferred_temperature_unit().suffix();
        Plot::new("Temperature distribution plot")
            .clamp_grid(true)
            .auto_bounds_x()
            .auto_bounds_y()
            .y_axis_label("% of image")
            .x_axis_label(format!(
                "Temperature ({})",
                global_state.preferred_temperature_unit().suffix()
            ))
            .y_axis_formatter(|factor, _max_chars, _range| format!("{:.0}%", factor))
            .x_axis_formatter(move |temp_val, _max_chars, _range| {
                return format!("{:.1} {}", temp_val, unit_suffix);
            })
            .show(ui, |plot_ui| plot_ui.bar_chart(chart));
    }
}
