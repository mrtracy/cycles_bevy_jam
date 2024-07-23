use bevy::prelude::*;
use bevy_egui::{
    egui::{self, Align, Align2, Layout, RichText},
    EguiContexts,
};

use crate::{plant_roots::Plant, GameState, Harvester, Score};

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

#[derive(Resource, Default, PartialEq)]
pub enum CurrentInspectedUnit {
    #[default]
    None,
    Tree(Entity),
    Harvester(Entity),
}

pub fn sys_selected_unit_ui(
    mut contexts: EguiContexts,
    current: Res<CurrentInspectedUnit>,
    tree_query: Query<&Plant>,
    harvester_query: Query<&Harvester>,
) {
    match *current {
        CurrentInspectedUnit::None => {}
        CurrentInspectedUnit::Tree(ent) => {
            let Ok(tree) = tree_query.get(ent) else {
                return;
            };
            egui::Window::new("Selected Unit")
                .collapsible(false)
                .anchor(Align2::LEFT_BOTTOM, egui::vec2(0.0, 0.0))
                .interactable(false)
                .resizable(false)
                .show(contexts.ctx_mut(), |ui| {
                    ui.label(format!("Tree type: {:?}", tree.genus));
                });
        }
        CurrentInspectedUnit::Harvester(ent) => {
            let Ok(harvester) = harvester_query.get(ent) else {
                return;
            };
            egui::Window::new("Selected Unit")
                .collapsible(false)
                .anchor(Align2::LEFT_BOTTOM, egui::vec2(0.0, 0.0))
                .interactable(false)
                .resizable(false)
                .show(contexts.ctx_mut(), |ui| {
                    ui.label(format!("Harvester, range: {}", harvester.range_units));
                });
        }
    }
}

// High scores
