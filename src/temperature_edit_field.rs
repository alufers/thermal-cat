use eframe::egui::{self, Response, Ui};

use crate::temperature::{TemperatureUnit, Temp};

pub fn temperature_edit_field(ui: &mut Ui, unit: TemperatureUnit, value: &mut Temp) -> Response {
    let mut tmp_value = format!("{}", value.to_unit(unit));
    let res = ui.text_edit_singleline(&mut tmp_value);
    if let Ok(result) = tmp_value.parse() {
        *value = Temp::from_unit(unit, result);
    }
    res
}
