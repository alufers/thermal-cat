
use eframe::egui;


pub trait Pane {
    fn title(&self) ->  egui::WidgetText;
    fn ui(&mut self, ui: &mut egui::Ui);
}

pub struct PaneDispatcher {}

///
/// Adapter from egui_dock::TabViewer to Pane
/// 
/// Makes it so we can implement each tab in a separate struct,
/// and dynamically dispatch to the correct code.\
/// 
impl egui_dock::TabViewer for PaneDispatcher {
    type Tab = Box<dyn Pane>;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.title().into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        tab.ui(ui);
    }
}
