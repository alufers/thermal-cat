use eframe::egui::{self, DragValue, Response, Ui};

use crate::temperature::{Temp, TempRange, TemperatureUnit};

pub fn temperature_edit_field(ui: &mut Ui, unit: TemperatureUnit, value: &mut Temp) -> Response {
    let mut tmp_value = value.to_unit(unit);
    let res = ui.add(
        DragValue::new(&mut tmp_value)
            .speed(0.5)
            .max_decimals(1)
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
    let mut did_change = false;
    let mut resp = egui::Grid::new(id_source)
        .show(ui, |ui| {
            if !enabled {
                ui.disable();
            }
            ui.label("Min");
            ui.label("Max");
            ui.end_row();
            if temperature_edit_field(ui, unit, &mut value.min).changed() {
                did_change = true;
                if value.min > value.max {
                    value.max = value.min;
                }
            }
            if temperature_edit_field(ui, unit, &mut value.max).changed() {
                did_change = true;
                if value.max < value.min {
                    value.min = value.max;
                }
            }
            ui.end_row();
        })
        .response;

    if did_change {
        resp.mark_changed();
    }
    resp
}
/// Edit a single emissivity value in the range 0.0 .. 1.0.
/// If the user drags past the bounds the value is clamped automatically.
pub fn emissivity_edit_field(ui: &mut Ui, value: &mut f32) -> Response {
    // make sure we start inside the bounds
    *value = value.clamp(0.0, 1.0);

    // the actual UI widget
    let mut v = *value;          // copy, because DragValue requires mutable reference
    let resp = ui.add(
        DragValue::new(&mut v)
            .speed(0.01)           // change step (adjust to your taste)
            .min_decimals(2)       // show two decimals by default
            .max_decimals(2)
            .clamp_range(0.0..=1.0) // <-- this guarantees value stays in range
    );

    // write back the possibly new value
    *value = v.clamp(0.0, 1.0);
    resp
}
