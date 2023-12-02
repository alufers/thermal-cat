use eframe::egui::{self, Response, Ui};

use crate::temperature_unit::TemperatureUnit;

pub fn temperature_edit_field(ui: &mut Ui, unit: TemperatureUnit, value_kelvin: &mut f32) -> Response {
    let mut tmp_value = format!("{}", unit.from_kelvin(*value_kelvin));
    let res = ui.text_edit_singleline(&mut tmp_value);
    if let Ok(result) = tmp_value.parse() {
        *value_kelvin = unit.to_kelvin(result);
    }
    res
}
