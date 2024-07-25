use std::any::TypeId;

use bevy::prelude::*;
use bevy_egui::{
    egui::{self, Align, Align2, Layout, RichText},
    EguiContexts,
};

use crate::{
    units::{harvester::HarvesterType, BuildingTypeMap, DebugPlantType},
    GameState, Score,
};

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
                    next_state.set(GameState::Loading);
                }
            });
        });
}

pub fn scoreboard(
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut score: ResMut<Score>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let score_label = format!("Score: {}", score.0);
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
            ui.menu_button("+", |ui| {
                if ui.button("Harvester").clicked() {
                    commands.insert_resource(CurrentIntention::Prospective(TypeId::of::<
                        HarvesterType,
                    >()))
                }
                if ui.button("DebugPlant").clicked() {
                    commands.insert_resource(CurrentIntention::Prospective(TypeId::of::<
                        DebugPlantType,
                    >()))
                }
            });
        });
}

#[derive(Resource, Default, PartialEq)]
pub enum CurrentIntention {
    #[default]
    None,
    Inspect(TypeId, Entity),
    Command(TypeId, Entity),
    Prospective(TypeId),
}

pub fn sys_selected_unit_ui(
    mut contexts: EguiContexts,
    current: Res<CurrentIntention>,
    building_types: Res<BuildingTypeMap>,
) {
    match *current {
        CurrentIntention::None => {}
        CurrentIntention::Inspect(type_id, _ent) => {
            let Some(building) = building_types.type_map.get(&type_id) else {
                return;
            };
            egui::Window::new("Selected Unit")
                .collapsible(false)
                .anchor(Align2::LEFT_BOTTOM, egui::vec2(0.0, 0.0))
                .interactable(false)
                .resizable(false)
                .show(contexts.ctx_mut(), |ui| {
                    ui.label(format!("Type: {}", building.name()));
                });
        }
        CurrentIntention::Command(type_id, _ent) => {
            let Some(building) = building_types.type_map.get(&type_id) else {
                return;
            };
            egui::Window::new("Command Unit")
                .collapsible(false)
                .anchor(Align2::LEFT_BOTTOM, egui::vec2(0.0, 0.0))
                .interactable(false)
                .resizable(false)
                .show(contexts.ctx_mut(), |ui| {
                    ui.label(format!("Type: {}", building.name()));
                });
        }
        CurrentIntention::Prospective(type_id) => {
            let Some(building) = building_types.type_map.get(&type_id) else {
                return;
            };
            egui::Window::new("Place Unit")
                .collapsible(false)
                .anchor(Align2::LEFT_BOTTOM, egui::vec2(0.0, 0.0))
                .interactable(false)
                .resizable(false)
                .show(contexts.ctx_mut(), |ui| {
                    ui.label(format!("Type: {}", building.name()));
                });
        }
    }
}

// High scores
