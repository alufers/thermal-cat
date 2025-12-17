use std::{cell::RefCell, rc::Rc};

use eframe::{egui, emath::Vec2b, epaint::Color32};
use egui_plot::{Bar, BarChart, Plot, VLine};

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

        let color_mapping_range = global_state
            .last_thermal_capturer_result
            .as_ref()
            .map(|r| r.image_range)
            .unwrap_or_else(|| global_state.thermal_capturer_settings.manual_range);

        let mut bucket_width = 1.0;
        if temperature_points.len() > 1 {
            bucket_width = (temperature_points[1].temperature - temperature_points[0].temperature)
                .to_unit(TemperatureUnit::Kelvin) as f64;
        }

        let chart = BarChart::new(
            "Histogram",
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
                        global_state.thermal_capturer_settings.temp_to_color(
                            p.temperature,
                            global_state
                                .last_thermal_capturer_result
                                .as_ref()
                                .map(|res| res.image_range),
                        ),
                    )
                })
                .collect(),
        );
        let unit_suffix = global_state.preferred_temperature_unit().suffix();

        Plot::new("Temperature distribution plot")
            .auto_bounds(Vec2b::TRUE)
            .y_axis_label("% of image")
            .x_axis_label(format!(
                "Temperature ({})",
                global_state.preferred_temperature_unit().suffix()
            ))
            .include_y(0.0)
            .include_y(30.0)
            .y_axis_formatter(|grid_mark, _range| format!("{:.0}%", grid_mark.value))
            .x_axis_formatter(move |grid_mark, _range| {
                format!("{:.0} {}", grid_mark.value, unit_suffix)
            })
            .show(ui, |plot_ui| {
                plot_ui.bar_chart(chart);
                if !color_mapping_range.is_default() {
                    plot_ui.vline(
                        VLine::new(
                            "Range line min",
                            color_mapping_range
                                .min
                                .to_unit(global_state.preferred_temperature_unit())
                                as f64,
                        )
                        .color(Color32::GRAY),
                    );
                    plot_ui.vline(
                        VLine::new(
                            "Range line max",
                            color_mapping_range
                                .max
                                .to_unit(global_state.preferred_temperature_unit())
                                as f64,
                        )
                        .color(Color32::GRAY),
                    );
                }
            });
    }
}
