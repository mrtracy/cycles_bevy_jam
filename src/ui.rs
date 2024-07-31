use std::any::TypeId;

use bevy::{prelude::*, render::render_resource::AsBindGroup};
use bevy_ecs_tilemap::{
    prelude::{MaterialTilemap, MaterialTilemapPlugin},
    tiles::TileColor,
};
use bevy_egui::{
    egui::{self, vec2, Align, Align2, Layout, RichText},
    EguiContexts,
};

use crate::{
    level::OverlayMaterialResource,
    nutrients::TileWater,
    units::{BuildingTypeMap, IntermissionTimer},
    AppState, GameType, PlayState, Score,
};

pub fn main_menu(mut contexts: EguiContexts, mut next_state: ResMut<NextState<AppState>>) {
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
                    next_state.set(AppState::Playing(GameType::NormalGame));
                }

                if ui
                    .button(RichText::new("Start Better").text_style(egui::TextStyle::Heading))
                    .clicked()
                {
                    next_state.set(AppState::Playing(GameType::TimsGame));
                }
            });
        });
}

pub fn scoreboard(
    mut contexts: EguiContexts,
    mut score: ResMut<Score>,
    mut overlay_mode: ResMut<OverlayMode>,
) {
    let score_label = format!("Score: {}", score.0);
    egui::Window::new("Fruitstar Score")
        .anchor(Align2::LEFT_BOTTOM, vec2(0.0, 0.0))
        .collapsible(false)
        .movable(false)
        .resizable(false)
        .fixed_size(vec2(150.0, 150.0))
        .show(contexts.ctx_mut(), |ui| {
            ui.set_width(ui.available_width());
            ui.set_height(ui.available_height());
            ui.label(RichText::new(score_label).text_style(egui::TextStyle::Heading));
            if ui.button("Toggle Water Overlay").clicked() {
                *overlay_mode = match *overlay_mode {
                    OverlayMode::Normal => OverlayMode::Water,
                    OverlayMode::Water => OverlayMode::Normal,
                }
            }
            if ui.button("Reset").clicked() {
                **score = 0;
            }
        });
}

pub fn sys_ui_build_board(
    mut contexts: EguiContexts,
    mut commands: Commands,
    building_types: Res<BuildingTypeMap>,
) {
    egui::Window::new("Units")
        .anchor(Align2::LEFT_BOTTOM, vec2(180.0, 0.0))
        .resizable(false)
        .collapsible(false)
        .movable(false)
        .fixed_size(vec2(450.0, 150.0))
        .show(contexts.ctx_mut(), |ui| {
            ui.set_width(ui.available_width());
            ui.set_height(ui.available_height());
            egui::Grid::new("tower_options")
                .num_columns(1)
                .show(ui, |ui| {
                    for (typ, built_type) in building_types.type_map.iter() {
                        if ui.button(built_type.name()).clicked() {
                            commands.insert_resource(CurrentIntention::Prospective(*typ));
                        }
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
                .anchor(Align2::RIGHT_TOP, egui::vec2(0.0, 0.0))
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
                .anchor(Align2::RIGHT_TOP, egui::vec2(0.0, 0.0))
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
                .anchor(Align2::RIGHT_TOP, egui::vec2(0.0, 0.0))
                .interactable(false)
                .resizable(false)
                .show(contexts.ctx_mut(), |ui| {
                    ui.label(format!("Type: {}", building.name()));
                });
        }
    }
}

#[derive(Component)]
pub struct UiTitleMessage;

pub fn sys_setup_ui_nodes(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Px(100.),
                align_content: AlignContent::Center,
                ..Default::default()
            },
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn((
                UiTitleMessage,
                TextBundle::from_section(
                    "FruitStart",
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        ..Default::default()
                    },
                ),
            ));
        });
}

#[derive(AsBindGroup, TypePath, Debug, Clone, Default, Asset)]
pub struct OverlayTilemapMaterial {
    #[uniform(0)]
    pub show_overlay: u32,
    // webgl2 requires 16 byte alignment
    #[uniform(0)]
    pub _padding: Vec3,
}

impl MaterialTilemap for OverlayTilemapMaterial {
    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        "tile_color_shader.wgsl".into()
    }
}

#[derive(Resource)]
pub enum OverlayMode {
    Normal,
    Water,
}

pub fn sys_show_overlay(
    mut tile_query: Query<(&mut TileColor, &TileWater)>,
    overlay_mode: Res<OverlayMode>,
    overlay_material: Res<OverlayMaterialResource>,
    mut materials: ResMut<Assets<OverlayTilemapMaterial>>,
) {
    match *overlay_mode {
        OverlayMode::Normal => {
            materials
                .get_mut(overlay_material.0.id())
                .unwrap()
                .show_overlay = 0;
            for (mut color, _) in &mut tile_query {
                if color.0 != Color::default() {
                    color.0 = Color::default()
                }
            }
        }
        OverlayMode::Water => {
            materials
                .get_mut(overlay_material.0.id())
                .unwrap()
                .show_overlay = 1;
            for (mut color, water_content) in &mut tile_query {
                color.0 = Color::linear_rgb(0.0, 0.0, (water_content.0 / 1000) as f32)
            }
        }
    }
}

pub fn sys_update_ui_title(
    mut query: Query<&mut Text, With<UiTitleMessage>>,
    play_state: Res<State<PlayState>>,
    intermission_timer: Res<IntermissionTimer>,
) {
    let Ok(mut text_node) = query.get_single_mut() else {
        warn!("UI title Text node not found");
        return;
    };
    match play_state.get() {
        PlayState::Intermission => {
            text_node.sections[0].value = format!(
                "Next Wave in {} secs",
                (intermission_timer.0.duration() - intermission_timer.0.elapsed()).as_secs_f32()
            );
        }
        PlayState::Setup => {
            text_node.sections[0].value = "".to_string();
        }
        PlayState::Wave => {
            text_node.sections[0].value = "Wave in Progress".to_string();
        }
        PlayState::Paused => {
            text_node.sections[0].value = "Game Paused".to_string();
        }
    }
}

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialTilemapPlugin::<OverlayTilemapMaterial>::default());
    }
}

// High scores
