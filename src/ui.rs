mod bottom_panel;
mod debug_window;
mod find_dialog;
pub mod inspect_panel;
mod top_panel;

use egui_sfml::{
    egui::{self, TopBottomPanel, Window},
    SfEgui,
};
use sfml::system::Vector2i;

use crate::app::App;

#[derive(Debug, Default)]
pub struct Ui {
    pub inspect_panel: InspectPanel,
    pub find_dialog: FindDialog,
    pub show_debug_panel: bool,
    pub fill_text: String,
    pub center_offset_input: String,
    pub seek_byte_offset_input: String,
}

use self::{find_dialog::FindDialog, inspect_panel::InspectPanel};

pub fn do_egui(sf_egui: &mut SfEgui, app: &mut App, mouse_pos: Vector2i) {
    sf_egui.do_frame(|ctx| {
        let mut open = app.ui.show_debug_panel;
        Window::new("Debug")
            .open(&mut open)
            .show(ctx, debug_window::ui);
        app.ui.show_debug_panel = open;
        open = app.ui.find_dialog.open;
        Window::new("Find")
            .open(&mut open)
            .show(ctx, |ui| FindDialog::ui(ui, app));
        app.ui.find_dialog.open = open;
        TopBottomPanel::top("top_panel").show(ctx, |ui| top_panel::ui(ui, app));
        TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| bottom_panel::ui(ui, app));
        egui::SidePanel::right("right_panel").show(ctx, |ui| inspect_panel::ui(ui, app, mouse_pos));
    });
}
