use bevy::prelude::*;
use bevy_egui::{
    egui::{self, Align, Layout, RichText},
    EguiContexts,
};

use crate::{GameState, Score};

pub fn main_menu(mut contexts: EguiContexts, mut next_state: ResMut<NextState<GameState>>) {
    egui::CentralPanel::default()
        .frame(egui::Frame::none())
        .show(contexts.ctx_mut(), |ui| {
            ui.add_space(40.0);
            ui.with_layout(Layout::top_down(Align::Center), |ui| {
                ui.heading("Welcome to Fruitstar!");
                ui.add_space(10.0);
                if ui
                    .button(RichText::new("Start").text_style(egui::TextStyle::Heading))
                    .clicked()
                {
                    next_state.set(GameState::Playing);
                }
            });
        });
}

pub fn scoreboard(
    mut contexts: EguiContexts,
    mut score: ResMut<Score>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let score_label = format!("Score: {}", score.to_string());
    egui::Window::new("Fruitstar")
        .collapsible(false)
        .movable(false)
        .interactable(false)
        .resizable(false)
        .show(contexts.ctx_mut(), |ui| {
            ui.label(RichText::new(score_label).text_style(egui::TextStyle::Heading));
            if ui.button("Reset").clicked() {
                **score = 0;
            }
            if ui.button("End").clicked() {
                next_state.set(GameState::GameOver);
            }
        });
}

// High scores
