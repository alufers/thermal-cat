use eframe::{
    egui::{
        load::{TextureLoadResult, TexturePoll},
        paint_texture_at, Image, ImageOptions, Rect, Response, Sense, Spinner, TextStyle, Ui,
        Widget, WidgetInfo, WidgetType,
    },
    emath::Align2,
    epaint::Vec2,
};

/// One out of several alternatives, either selected or not.
/// Will mark selected items with a different background color.
/// An alternative to [`RadioButton`] and [`Checkbox`].
///
/// Usually you'd use [`Ui::selectable_value`] or [`Ui::selectable_label`] instead.
///
/// ```
/// # egui::__run_test_ui(|ui| {
/// #[derive(PartialEq)]
/// enum Enum { First, Second, Third }
/// let mut my_enum = Enum::First;
///
/// ui.selectable_value(&mut my_enum, Enum::First, "First");
///
/// // is equivalent to:
///
/// if ui.add(egui::SelectableImageLabel::new(my_enum == Enum::First, "First")).clicked() {
///     my_enum = Enum::First
/// }
/// # });
/// ```
#[must_use = "You should put this widget in an ui with `ui.add(widget);`"]
pub struct SelectableImageLabel<'a> {
    selected: bool,
    image: Image<'a>,
}

impl<'a> SelectableImageLabel<'a> {
    pub fn new(selected: bool, image: impl Into<Image<'a>>) -> Self {
        Self {
            selected,
            image: image.into(),
        }
    }
}

impl<'a> Widget for SelectableImageLabel<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let Self {
            selected,
            ref image,
        } = self;

        let button_padding = Vec2::splat(ui.spacing().button_padding.x);
        let available_size_for_image = ui.available_size() - 2.0 * button_padding;
        let tlr = self.image.load_for_size(ui.ctx(), available_size_for_image);
        let original_image_size = tlr.as_ref().ok().and_then(|t| t.size());
        let image_size = self
            .image
            .calc_size(available_size_for_image, original_image_size);

        let padded_size = image_size + 2.0 * button_padding;
        let (rect, response) = ui.allocate_exact_size(padded_size, Sense::click());
        response.widget_info(|| WidgetInfo::new(WidgetType::ImageButton));

        if ui.is_rect_visible(response.rect) {
            let _text_pos = ui
                .layout()
                .align_size_within_rect(
                    image.size().unwrap_or_default(),
                    rect.shrink2(button_padding),
                )
                .min;

            let visuals = ui.style().interact_selectable(&response, selected);

            if selected || response.hovered() || response.highlighted() || response.has_focus() {
                let rect = rect.expand(visuals.expansion);

                ui.painter().rect(
                    rect,
                    visuals.rounding,
                    visuals.weak_bg_fill,
                    visuals.bg_stroke,
                );
            }

            let image_rect = ui
                .layout()
                .align_size_within_rect(image_size, rect.shrink2(button_padding));
            // let image_rect = image_rect.expand2(expansion); // can make it blurry, so let's not
            let image_options = image.image_options().clone();

            paint_texture_load_result(ui, &tlr, image_rect, None, &image_options);
        }

        response
    }
}

pub fn paint_texture_load_result(
    ui: &Ui,
    tlr: &TextureLoadResult,
    rect: Rect,
    show_loading_spinner: Option<bool>,
    options: &ImageOptions,
) {
    match tlr {
        Ok(TexturePoll::Ready { texture }) => {
            paint_texture_at(ui.painter(), rect, options, texture);
        }
        Ok(TexturePoll::Pending { .. }) => {
            let show_loading_spinner =
                show_loading_spinner.unwrap_or(ui.visuals().image_loading_spinners);
            if show_loading_spinner {
                Spinner::new().paint_at(ui, rect);
            }
        }
        Err(_) => {
            let font_id = TextStyle::Body.resolve(ui.style());
            ui.painter().text(
                rect.center(),
                Align2::CENTER_CENTER,
                "⚠",
                font_id,
                ui.visuals().error_fg_color,
            );
        }
    }
}
