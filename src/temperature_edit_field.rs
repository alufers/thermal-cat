use eframe::egui::{self, DragValue, Response, Ui};

use crate::temperature::{Temp, TempRange, TemperatureUnit};

pub fn temperature_edit_field(ui: &mut Ui, unit: TemperatureUnit, value: &mut Temp) -> Response {
    let mut tmp_value = value.to_unit(unit);
    let res = ui.add(
        DragValue::new(&mut tmp_value)
            .speed(0.5)
            .suffix(unit.suffix()),
    );
    *value = Temp::from_unit(unit, tmp_value);
    res
}

pub fn temperature_range_edit_field(
    ui: &mut Ui,
    id_source: impl std::hash::Hash,
    enabled: bool,
    unit: TemperatureUnit,
    value: &mut TempRange,
) -> Response {
    egui::Grid::new(id_source)
        .show(ui, |ui| {
            ui.set_enabled(enabled);
            ui.label("Min");
            ui.label("Max");
            ui.end_row();
            if temperature_edit_field(ui, unit, &mut value.min).changed() {
                if value.min > value.max {
                    value.max = value.min;
                }
            }
            if temperature_edit_field(ui, unit, &mut value.max).changed() {
                if value.max < value.min {
                    value.min = value.max;
                }
            }
            ui.end_row();
        })
        .response
}
