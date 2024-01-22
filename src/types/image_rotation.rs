#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ImageRotation {
    None,
    Clockwise90,
    Clockwise180,
    Clockwise270,
}

impl ImageRotation {
    pub fn next(self) -> Self {
        match self {
            Self::None => Self::Clockwise90,
            Self::Clockwise90 => Self::Clockwise180,
            Self::Clockwise180 => Self::Clockwise270,
            Self::Clockwise270 => Self::None,
        }
    }
    pub fn prev(self) -> Self {
        match self {
            Self::None => Self::Clockwise270,
            Self::Clockwise90 => Self::None,
            Self::Clockwise180 => Self::Clockwise90,
            Self::Clockwise270 => Self::Clockwise180,
        }
    }
}
