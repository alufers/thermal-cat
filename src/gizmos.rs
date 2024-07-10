use eframe::epaint::{Color32, Hsva};
use uuid::Uuid;

use crate::{temperature::Temp, thermal_data::ThermalDataPos};

#[derive(Clone)]
pub enum GizmoKind {
    Root { children: Vec<Gizmo> },
    MaxTemp,
    MinTemp,
    TempAt { pos: ThermalDataPos },
}

#[derive(Clone)]
pub struct Gizmo {
    pub uuid: Uuid,
    pub kind: GizmoKind,
    pub name: String,
    pub color: Color32,
    pub show_temperature_label: bool,
}

impl Gizmo {
    pub fn new(kind: GizmoKind, name: String, color: Color32) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            kind,
            name,
            color,
            show_temperature_label: true,
        }
    }
    pub fn new_root(children: Vec<Gizmo>) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            kind: GizmoKind::Root { children },
            name: "Root".to_string(),
            color: Color32::WHITE,
            show_temperature_label: true,
        }
    }

    pub fn children_mut(&mut self) -> Option<&mut Vec<Gizmo>> {
        match &mut self.kind {
            GizmoKind::Root { children } => Some(children),
            _ => None,
        }
    }

    pub fn push_child(&mut self, kind: GizmoKind, name: String) {
        match &mut self.kind {
            GizmoKind::Root { children } => {
                let last_child_color = children
                    .last()
                    .map(|c| c.color)
                    .unwrap_or(Color32::from_rgb(255, 0, 0));
                let mut new_color = Hsva::from(last_child_color);
                new_color.h += 0.1;
                children.push(Gizmo::new(kind, name, new_color.into()));
            }
            _ => panic!("Cannot push child to non-root gizmo"),
        }
    }
}

#[derive(Clone)]
pub struct GizmoResult {
    pub temperature: Temp,
    pub pos: ThermalDataPos,
}
