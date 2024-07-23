use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

pub fn example(mut contexts: EguiContexts) {
    egui::Window::new("Hello").show(contexts.ctx_mut(), |ui| {
        ui.label("world");
    });
}
