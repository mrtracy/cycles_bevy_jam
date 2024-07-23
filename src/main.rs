// Bevy code commonly triggers these lints and they may be important signals
// about code quality. They are sometimes hard to avoid though, and the CI
// workflow treats them as errors, so this allows them throughout the project.
// Feel free to delete this line.
#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use std::time::Duration;

use bevy::asset::AssetMetaCheck;
use bevy::math::vec2;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::EguiPlugin;
use bevy_spatial::kdtree::KDTree2;
use bevy_spatial::{AutomaticUpdate, SpatialAccess, SpatialStructure, TransformMode};
use fruit::{FruitBranch, FruitBranchBundle};
use fruit_type::FruitSpeciesPlugin;

mod fruit;
mod fruit_type;
mod plant_roots;
mod ui;

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
    #[default]
    MainMenu,
    Playing,
    GameOver,
}

#[derive(Component, Default)]
pub struct SpatialTracked;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            // Wasm builds will check for meta files (that don't exist) if this isn't set.
            // This causes errors and even panics in web builds on itch.
            // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
            meta_check: AssetMetaCheck::Never,
            ..default()
        }))
        .add_plugins(
            AutomaticUpdate::<SpatialTracked>::new()
                .with_frequency(Duration::from_secs_f32(0.3))
                .with_spatial_ds(SpatialStructure::KDTree2)
                .with_transform(TransformMode::GlobalTransform),
        )
        .add_plugins(EguiPlugin)
        .add_plugins(FruitSpeciesPlugin)
        .insert_resource(LevelBounds {
            min: vec2(-300.0, -300.0),
            max: vec2(300.0, 300.0),
        })
        .insert_resource(Score(0))
        .init_state::<GameState>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                (ui::main_menu).run_if(in_state(GameState::MainMenu)),
                (
                    sys_draw_border,
                    sys_spawn_on_click,
                    plant_roots::sys_plant_move,
                    sys_harvester_look_for_fruit,
                    sys_harvester_target_set,
                    sys_harvester_move_to_target,
                    fruit::sys_fruit_branch_spawn_fruit,
                    fruit::sys_fruit_grow,
                    ui::scoreboard,
                )
                    .run_if(in_state(GameState::Playing)),
            ),
        )
        .add_systems(OnEnter(GameState::Playing), setup_game)
        .observe(fruit::obs_fruit_harvested)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn setup_game(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Harvester::new_bundle(&asset_server, 50));
}

pub fn sys_draw_border(mut gizmos: Gizmos, bounds: Res<LevelBounds>) {
    gizmos.line_2d(bounds.min, bounds.max.with_y(bounds.min.y), Color::WHITE);
    gizmos.line_2d(bounds.min, bounds.max.with_x(bounds.min.x), Color::WHITE);
    gizmos.line_2d(bounds.max, bounds.max.with_y(bounds.min.y), Color::WHITE);
    gizmos.line_2d(bounds.max, bounds.max.with_x(bounds.min.x), Color::WHITE);
}

pub fn sys_harvester_look_for_fruit(
    mut commands: Commands,
    spatial_tree: Res<KDTree2<SpatialTracked>>,
    harvesters: Query<(Entity, &Harvester, &Transform)>,
    fruit: Query<&fruit::FruitGrowthState>,
) {
    for (harvester_ent, harvester, transform) in harvesters.iter() {
        for (_, entity) in spatial_tree.within_distance(
            transform.translation.xy() + vec2(16.0, 16.0),
            harvester.range_units as f32,
        ) {
            let Some(entity) = entity else { continue };
            let Ok(fruit::FruitGrowthState::Fruited) = fruit.get(entity) else {
                continue;
            };
            commands.trigger_targets(fruit::HarvestFruitEvent { harvester_ent }, entity);
        }
    }
}

pub fn get_world_click_pos(
    q_windows: &Query<&Window, With<PrimaryWindow>>,
    camera: &Query<(&Camera, &GlobalTransform)>,
) -> Option<Vec2> {
    let Some(pos) = q_windows
        .get_single()
        .ok()
        .and_then(|w| w.cursor_position())
    else {
        return None;
    };
    let Ok((camera, gt)) = camera.get_single() else {
        return None;
    };
    camera.viewport_to_world_2d(gt, pos)
}

pub fn sys_spawn_on_click(
    mut commands: Commands,
    buttons: Res<ButtonInput<MouseButton>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
    asset_server: Res<AssetServer>,
    bounds: Res<LevelBounds>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        if let Some(pos) = get_world_click_pos(&q_windows, &camera) {
            if bounds.in_bounds(pos) {
                commands
                    .spawn(plant_roots::Plant::new_bundle(
                        asset_server.load("plant_base_test.png"),
                        pos,
                    ))
                    .with_children(|child_commands| {
                        child_commands.spawn(FruitBranchBundle {
                            branch: FruitBranch { species: 0 },
                            sprite: SpriteBundle {
                                ..Default::default()
                            },
                        });
                    });
            }
        }
    }
}

#[derive(Resource)]
pub struct LevelBounds {
    pub min: Vec2,
    pub max: Vec2,
}

impl LevelBounds {
    pub fn in_bounds(&self, point: Vec2) -> bool {
        self.min.x < point.x && self.min.y < point.y && self.max.x > point.x && self.max.y > point.y
    }
}

#[derive(Component)]
pub struct Harvester {
    pub range_units: usize,
    pub target: Vec2,
}

impl Harvester {
    pub fn new_bundle(asset_server: &AssetServer, range_units: usize) -> impl Bundle {
        (
            Harvester {
                range_units,
                target: vec2(0., 0.),
            },
            SpriteBundle {
                texture: asset_server.load("harvester_test.png"),
                ..Default::default()
            },
        )
    }
}

pub fn sys_harvester_target_set(
    buttons: Res<ButtonInput<MouseButton>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
    mut harvester: Query<(&Transform, &mut Harvester)>,
    bounds: Res<LevelBounds>,
    mut gizmos: Gizmos,
) {
    if buttons.pressed(MouseButton::Right) {
        if let Some(pos) = get_world_click_pos(&q_windows, &camera) {
            if bounds.in_bounds(pos) {
                harvester
                    .get_single_mut()
                    .map(|(_, mut h)| h.as_mut().target = vec2(pos.x, pos.y))
                    .expect("Failed to update target");
                gizmos.arrow_2d(harvester.single().0.translation.xy(), pos, Color::WHITE);
            }
        }
    }
}

pub fn sys_harvester_move_to_target(
    mut harvester: Query<(&mut Transform, &Harvester)>,
    time: Res<Time>,
) {
    let (mut transform, harvester) = harvester.single_mut();
    let current_translation = transform.translation;
    transform.translation =
        current_translation.move_towards(harvester.target.extend(0.), 50.0 * time.delta_seconds());
}

#[derive(Resource, Deref, DerefMut)]
pub struct Score(usize);
