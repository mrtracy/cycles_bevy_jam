use bevy::prelude::*;
use bevy_egui::{
    egui::{self, RichText},
    EguiContexts,
};

use crate::Score;

pub fn scoreboard(mut contexts: EguiContexts, score: Res<Score>) {
    let score_label = format!("Score: {}", score.to_string());
    egui::Window::new("Fruitstar")
        .collapsible(false)
        .movable(false)
        .interactable(false)
        .resizable(false)
        .show(contexts.ctx_mut(), |ui| {
            ui.label(RichText::new(score_label).text_style(egui::TextStyle::Heading));
        });
}
