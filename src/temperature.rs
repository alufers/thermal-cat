use serde::{Deserialize, Serialize};
// 0.17.1
use strum_macros::{Display, EnumIter}; // 0.17.1

use std::{
    fmt::{self, Debug},
    ops,
};

//
// Represents a temperature in Kelvin.
//
#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct Temp {
    value_kelvin: f32,
}

impl Temp {
    pub const MAX: Temp = Temp {
        value_kelvin: f32::MAX,
    };
    pub const MIN: Temp = Temp {
        value_kelvin: f32::MIN,
    };
    pub fn new(value_kelvin: f32) -> Self {
        Self { value_kelvin }
    }

    pub fn from_celsius(value: f32) -> Self {
        Temp::from_unit(TemperatureUnit::Celsius, value)
    }

    pub fn from_unit(unit: TemperatureUnit, value: f32) -> Self {
        Self {
            value_kelvin: match unit {
                TemperatureUnit::Kelvin => value,
                TemperatureUnit::Celsius => value + 273.15,
                TemperatureUnit::Fahrenheit => (value - 32.0) / 1.8 + 273.15,
            },
        }
    }
    pub fn to_unit(self, unit: TemperatureUnit) -> f32 {
        match unit {
            TemperatureUnit::Kelvin => self.value_kelvin,
            TemperatureUnit::Celsius => self.value_kelvin - 273.15,
            TemperatureUnit::Fahrenheit => (self.value_kelvin - 273.15) * 1.8 + 32.0,
        }
    }
}

impl Default for Temp {
    fn default() -> Self {
        Self { value_kelvin: 0.0 }
    }
}

impl Debug for Temp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let debug_unit = TemperatureUnit::Celsius;
        write!(f, "{} {}", self.to_unit(debug_unit), debug_unit.suffix())
    }
}

impl ops::Add for Temp {
    type Output = Temp;

    fn add(self, rhs: Temp) -> Self::Output {
        Temp {
            value_kelvin: self.value_kelvin + rhs.value_kelvin,
        }
    }
}

impl ops::Sub for Temp {
    type Output = Temp;

    fn sub(self, rhs: Temp) -> Self::Output {
        Temp {
            value_kelvin: self.value_kelvin - rhs.value_kelvin,
        }
    }
}

impl ops::Mul<f32> for Temp {
    type Output = Temp;

    fn mul(self, rhs: f32) -> Self::Output {
        Temp {
            value_kelvin: self.value_kelvin * rhs,
        }
    }
}

impl ops::Div<f32> for Temp {
    type Output = Temp;

    fn div(self, rhs: f32) -> Self::Output {
        Temp {
            value_kelvin: self.value_kelvin / rhs,
        }
    }
}

impl ops::Div<Temp> for Temp {
    // Dividing two temperatures gives a ratio, which is a unitless value.
    type Output = f32;

    fn div(self, rhs: Temp) -> Self::Output {
        self.value_kelvin / rhs.value_kelvin
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct TempRange {
    pub min: Temp,
    pub max: Temp,
}

impl Default for TempRange {
    fn default() -> Self {
        Self {
            min: Temp::MIN,
            max: Temp::MAX,
        }
    }
}

impl TempRange {
    pub fn new(min: Temp, max: Temp) -> Self {
        Self { min, max }
    }

    pub fn factor(&self, temp: Temp) -> f32 {
        (temp.value_kelvin - self.min.value_kelvin)
            / (self.max.value_kelvin - self.min.value_kelvin)
    }

    pub fn contains(&self, temp: Temp) -> bool {
        temp.value_kelvin >= self.min.value_kelvin && temp.value_kelvin <= self.max.value_kelvin
    }

    pub fn contains_range(&self, range: TempRange) -> bool {
        self.contains(range.min) && self.contains(range.max)
    }

    pub fn animate(&self, target: TempRange, factor: f32) -> TempRange {
        TempRange {
            min: Temp::new(
                self.min.value_kelvin + (target.min.value_kelvin - self.min.value_kelvin) * factor,
            ),
            max: Temp::new(
                self.max.value_kelvin + (target.max.value_kelvin - self.max.value_kelvin) * factor,
            ),
        }
    }

    pub fn diff(&self) -> Temp {
        self.max - self.min
    }

    pub fn join(&self, other: TempRange) -> TempRange {
        TempRange {
            min: Temp::new(self.min.value_kelvin.min(other.min.value_kelvin)),
            max: Temp::new(self.max.value_kelvin.max(other.max.value_kelvin)),
        }
    }

    ///
    /// Returns true if this range is the default range, which is the entire range of possible
    /// temperatures.
    ///
    pub fn is_default(&self) -> bool {
        self.min == Temp::MIN && self.max == Temp::MAX
    }
}

impl Debug for TempRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{:?} - {:?}]", self.min, self.max)
    }
}

#[derive(EnumIter, Display, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TemperatureUnit {
    #[default]
    Kelvin,
    Celsius,
    Fahrenheit,
}

impl TemperatureUnit {
    pub fn suffix(&self) -> String {
        match self {
            TemperatureUnit::Kelvin => "K".to_string(),
            TemperatureUnit::Celsius => "°C".to_string(),
            TemperatureUnit::Fahrenheit => "°F".to_string(),
        }
    }
}
