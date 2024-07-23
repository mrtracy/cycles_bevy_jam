use bevy_egui::{
    egui::{self, RichText},
    EguiContexts,
};

pub fn example(mut contexts: EguiContexts) {
    egui::Window::new("Fruitstar")
        .collapsible(false)
        .movable(false)
        .interactable(false)
        .resizable(false)
        .show(contexts.ctx_mut(), |ui| {
            ui.label(RichText::new("Score: ∞").text_style(egui::TextStyle::Body));
        });
}
