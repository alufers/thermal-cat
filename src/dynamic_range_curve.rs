use std::hash::{DefaultHasher, Hash, Hasher};

use eframe::{
    egui::{self, CursorIcon, Id, Image, ImageButton, Layout, TextureOptions, Ui},
    emath::{Align, Vec2b},
    epaint::Vec2,
};
use egui_plot::{Line, MarkerShape, Plot, PlotBounds, PlotImage, PlotPoint, PlotPoints, Points};

use crate::{
    temperature::{TempRange, TemperatureUnit},
    thermal_capturer::ThermalCapturerSettings,
    util::{rotate_image, ImageRotation},
};

#[derive(Clone, Debug, PartialEq)]
pub enum CurvePoint {
    Sharp(f32, f32),
    Smooth(f32, f32),
}

impl CurvePoint {
    pub fn x(&self) -> f32 {
        match self {
            CurvePoint::Sharp(x, _) => *x,
            CurvePoint::Smooth(x, _) => *x,
        }
    }

    pub fn y(&self) -> f32 {
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
                *x = pos.x;
                *y = pos.y;
            }
            CurvePoint::Smooth(x, y) => {
                *x = pos.x;
                *y = pos.y;
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
    pub fn get_value(&self, x: f32) -> f32 {
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

#[derive(Default, Clone)]
struct CurveEditorState {
    dragged_point_idx: Option<usize>,
    ref_gradient_tex: Option<egui::TextureHandle>,
    last_gradient_hash: u64,
}

#[derive(Clone, Debug, Default)]
pub struct CurveEditorResponse {
    changed: bool,
}
impl CurveEditorResponse {
    pub fn changed(&self) -> bool {
        self.changed
    }
}

pub fn dynamic_curve_editor(
    ui: &mut Ui,
    id: impl std::hash::Hash,
    settings: &mut ThermalCapturerSettings,
    current_range: TempRange,
    unit: TemperatureUnit,
) -> CurveEditorResponse {
    let mut response = CurveEditorResponse::default();
    let memory_id = Id::new(id);

    let curve = &mut settings.dynamic_range_curve;
    let gradient = &settings.gradient;

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
                response.changed = true;
            }
        },
    );
    Plot::new(memory_id.with("plot"))
        .show_axes(Vec2b::new(true, false))
        .allow_drag(false)
        .allow_zoom(false)
        .allow_double_click_reset(false)
        .allow_scroll(false)
        .allow_boxed_zoom(false)
        .show_x(false)
        .show_y(false)
        .height(250.0)
        // .data_aspect(1.0)
        // .view_aspect(1.0)
        .x_axis_formatter(move |x, _, _| {
            format!(
                "{:.0} {}",
                current_range.factor_to_temp(x as f32).to_unit(unit),
                unit.suffix()
            )
        })
        .show(ui, |plot_ui| {
            let mut state = plot_ui
                .ctx()
                .memory(|mem| mem.data.get_temp::<CurveEditorState>(memory_id))
                .unwrap_or_default();

            let mut state_dirty = false;

            // generate refgerence gradient texture if needed
            let mut hasher = DefaultHasher::new();
            gradient.hash(&mut hasher);
            let gradient_hash: u64 = hasher.finish();
            if state.last_gradient_hash != gradient_hash {
                state.last_gradient_hash = gradient_hash;
                state.ref_gradient_tex = Some(plot_ui.ctx().load_texture(
                    "curve_editor_ref_gradient",
                    rotate_image(
                        gradient.create_demo_image(128, 2),
                        ImageRotation::Clockwise90,
                    ),
                    TextureOptions {
                        ..Default::default()
                    },
                ));
                state_dirty = true;
            }

            plot_ui.set_plot_bounds(PlotBounds::from_min_max([-0.05, -0.05], [1.05, 1.05]));

            // draw line
            let n = plot_ui.response().rect.width() as i32 / 4;
            let line_points: PlotPoints = (0..=n)
                .map(|i| {
                    let x = i as f32 / n as f32;
                    [x as f64, curve.get_value(x) as f64]
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
                        let p = CurvePoint::Sharp(pointer_pos.x as f32, pointer_pos.y as f32);
                        response.changed = true;
                        curve.insert_point_at(p)
                    })
                });
                state_dirty = true;
            }

            if let Some(drag_idx) = state.dragged_point_idx {
                match curve.points.get(drag_idx) {
                    Some(point) => {
                        if plot_ui.pointer_coordinate_drag_delta().length() > f32::EPSILON {
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
                            response.changed = true;
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
                let is_dragged = state.dragged_point_idx == Some(i);
                let border_color = if let Some(hovered_idx) = hovered_point_idx
                    && i == hovered_idx
                {
                    plot_ui
                        .ctx()
                        .style()
                        .visuals
                        .widgets
                        .hovered
                        .fg_stroke
                        .color
                } else {
                    plot_ui
                        .ctx()
                        .style()
                        .visuals
                        .widgets
                        .inactive
                        .fg_stroke
                        .color
                };
                if is_dragged {
                    plot_ui.points(
                        Points::new(vec![[p.x() as f64, p.y() as f64]])
                            .shape(match p {
                                CurvePoint::Sharp(_, _) => MarkerShape::Square,
                                CurvePoint::Smooth(_, _) => MarkerShape::Circle,
                            })
                            .color(plot_ui.ctx().style().visuals.selection.bg_fill)
                            .filled(true)
                            .radius(5.0),
                    );
                }

                plot_ui.points(
                    Points::new(vec![[p.x() as f64, p.y() as f64]])
                        .shape(match p {
                            CurvePoint::Sharp(_, _) => MarkerShape::Square,
                            CurvePoint::Smooth(_, _) => MarkerShape::Circle,
                        })
                        .color(border_color)
                        .filled(false)
                        .radius(5.0),
                );
            }

            // draw reference gradient

            plot_ui.image(PlotImage::new(
                state
                    .ref_gradient_tex
                    .as_ref()
                    .expect("ref_gradient_tex not set"),
                PlotPoint::new(-0.5, 0.5),
                Vec2::new(1.0, 1.0),
            ));

            // persist state if dirty
            if state_dirty {
                let id_clone = memory_id;
                let state_clone = state.clone();

                plot_ui
                    .ctx()
                    .memory_mut(|mem| mem.data.insert_temp(id_clone, state_clone));
            }
        });
    response
}
