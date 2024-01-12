use eframe::{
    egui::{self, CursorIcon, Id, Image, ImageButton, Layout, Ui},
    emath::Align,
    epaint::{Color32, Vec2},
};
use egui_plot::{Line, MarkerShape, Plot, PlotPoints, Points};

#[derive(Clone, Debug, PartialEq)]
pub enum CurvePoint {
    Sharp(f64, f64),
    Smooth(f64, f64),
}

impl CurvePoint {
    pub fn x(&self) -> f64 {
        match self {
            CurvePoint::Sharp(x, _) => *x,
            CurvePoint::Smooth(x, _) => *x,
        }
    }

    pub fn y(&self) -> f64 {
        match self {
            CurvePoint::Sharp(_, y) => *y,
            CurvePoint::Smooth(_, y) => *y,
        }
    }

    pub fn pos(&self) -> Vec2 {
        Vec2::new(self.x() as f32, self.y() as f32)
    }

    pub fn set_pos(&mut self, pos: Vec2) {
        match self {
            CurvePoint::Sharp(x, y) => {
                *x = pos.x as f64;
                *y = pos.y as f64;
            }
            CurvePoint::Smooth(x, y) => {
                *x = pos.x as f64;
                *y = pos.y as f64;
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct DynamicRangeCurve {
    pub points: Vec<CurvePoint>,
}

impl Default for DynamicRangeCurve {
    fn default() -> Self {
        Self {
            points: vec![CurvePoint::Sharp(0.0, 0.0), CurvePoint::Sharp(1.0, 1.0)],
        }
    }
}

impl DynamicRangeCurve {
    pub fn is_default(&mut self) -> bool {
        self.points.len() == 2
            && self.points[0] == CurvePoint::Sharp(0.0, 0.0)
            && self.points[1] == CurvePoint::Sharp(1.0, 1.0)
    }
    pub fn get_value(&self, x: f64) -> f64 {
        for i in 0..self.points.len() - 1 {
            let p1 = &self.points[i];
            let p2 = &self.points[i + 1];
            if p1.x() <= x && x <= p2.x() {
                let t = (x - p1.x()) / (p2.x() - p1.x());
                return p1.y() * (1.0 - t) + p2.y() * t;
            }
        }
        if !self.points.is_empty() {
            let first = &self.points[0];
            if x < first.x() {
                first.y()
            } else {
                let last = &self.points[self.points.len() - 1];
                last.y()
            }
        } else {
            x
        }
    }

    // Insert a point at the correct position
    // Returns the index of the inserted point
    pub fn insert_point_at(&mut self, p: CurvePoint) -> usize {
        let mut insert_idx = None;
        for i in 0..self.points.len() - 1 {
            let p1 = &self.points[i];
            let p2 = &self.points[i + 1];
            if p1.x() <= p.x() && p.x() <= p2.x() {
                insert_idx = Some(i + 1);
                break;
            }
        }
        if let Some(idx) = insert_idx {
            self.points.insert(idx, p);
            idx
        } else {
            self.points.push(p);
            self.points.len() - 1
        }
    }
}

#[derive(Default, Clone, Debug)]
struct CurveEditorState {
    dragged_point_idx: Option<usize>,
}

pub fn dynamic_curve_editor(ui: &mut Ui, id: impl std::hash::Hash, curve: &mut DynamicRangeCurve) {
    let memory_id = Id::new(id);

    ui.with_layout(
        Layout::right_to_left(Align::Min).with_cross_justify(false),
        |ui| {
            if ui
                .add_enabled(
                    !curve.is_default(),
                    ImageButton::new(
                        Image::new(egui::include_image!("./icons/rotate-ccw.svg")).max_height(16.0),
                    ),
                )
                .clicked()
            {
                *curve = DynamicRangeCurve::default();
            }
        },
    );
    Plot::new(memory_id.with("plot"))
        .show_axes(false)
        .allow_drag(false)
        .allow_zoom(false)
        .allow_double_click_reset(false)
        .allow_scroll(false)
        .allow_boxed_zoom(false)
        .show_x(false)
        .show_y(false)
        .data_aspect(1.0)
        .view_aspect(1.0)
        .include_x(0.0)
        .include_y(0.0)
        .include_x(1.0)
        .include_y(1.0)
        .show(ui, |plot_ui| {
            let mut state = plot_ui
                .ctx()
                .memory(|mem| mem.data.get_temp::<CurveEditorState>(memory_id))
                .unwrap_or_default();
            let mut state_dirty = false;

            let n = plot_ui.response().rect.width() as i32 / 4;
            let line_points: PlotPoints = (0..=n)
                .map(|i| {
                    let x = i as f64 / n as f64;
                    [x, curve.get_value(x)]
                })
                .collect();
            plot_ui.line(Line::new(line_points));

            // determine hovered point
            let hover_dist: f32 = (1.0 / plot_ui.transform().dpos_dvalue_x().abs() * 20.0) as f32;
            let hovered_point_idx: Option<usize> =
                plot_ui.pointer_coordinate().and_then(|cursor_pos| {
                    for (i, p) in curve.points.iter().enumerate() {
                        if (p.pos() - cursor_pos.to_vec2()).length() < hover_dist {
                            return Some(i);
                        }
                    }
                    None
                });

            if hovered_point_idx.is_some() {
                plot_ui
                    .ctx()
                    .output_mut(|out| out.cursor_icon = CursorIcon::Grab);
            }
            if plot_ui.response().drag_started() {
                state.dragged_point_idx = hovered_point_idx.or_else(|| {
                    plot_ui.pointer_coordinate().map(|pointer_pos| {
                        let p = CurvePoint::Sharp(pointer_pos.x, pointer_pos.y);
                        curve.insert_point_at(p)
                    })
                });
                state_dirty = true;
            }

            if let Some(drag_idx) = state.dragged_point_idx {
                match curve.points.get(drag_idx) {
                    Some(point) => {
                        let new_pos = (point.pos() + plot_ui.pointer_coordinate_drag_delta())
                            .clamp(Vec2::ZERO, Vec2::splat(1.0));
                        let exceeds_other_points = curve
                            .points
                            .get(drag_idx.wrapping_sub(1)) // if it wraps around, it's fine
                            .map(|f| new_pos.x < (f.x() as f32))
                            .unwrap_or_default()
                            || curve
                                .points
                                .get(drag_idx + 1)
                                .map(|f| new_pos.x > (f.x() as f32))
                                .unwrap_or_default();
                        if !exceeds_other_points {
                            curve.points[drag_idx].set_pos(new_pos);
                        } else {
                            // user has dragged the point to far, remove it
                            curve.points.remove(drag_idx);
                            state.dragged_point_idx = None;
                            state_dirty = true;
                        }
                    }
                    None => {
                        state.dragged_point_idx = None;
                        state_dirty = true;
                    }
                }
            }

            // draw point markers
            for (i, p) in curve.points.iter().enumerate() {
                let color = if let Some(hovered_idx) = hovered_point_idx
                    && i == hovered_idx
                {
                    Color32::RED
                } else {
                    Color32::WHITE
                };

                plot_ui.points(
                    Points::new(vec![[p.x(), p.y()]])
                        .shape(match p {
                            CurvePoint::Sharp(_, _) => MarkerShape::Square,
                            CurvePoint::Smooth(_, _) => MarkerShape::Circle,
                        })
                        .color(color)
                        .filled(state.dragged_point_idx == Some(i))
                        .radius(5.0),
                );
            }

            // persist state if dirty
            if state_dirty {
                let id_clone = memory_id;
                let state_clone = state.clone();

                plot_ui
                    .ctx()
                    .memory_mut(|mem| mem.data.insert_temp(id_clone, state_clone));
            }
        });
}
