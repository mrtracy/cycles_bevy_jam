// Bevy code commonly triggers these lints and they may be important signals
// about code quality. They are sometimes hard to avoid though, and the CI
// workflow treats them as errors, so this allows them throughout the project.
// Feel free to delete this line.
#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use bevy::asset::AssetMetaCheck;
use bevy::math::vec2;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            // Wasm builds will check for meta files (that don't exist) if this isn't set.
            // This causes errors and even panics in web builds on itch.
            // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
            meta_check: AssetMetaCheck::Never,
            ..default()
        }))
        .insert_resource(LevelBounds {
            min: vec2(-300.0, -300.0),
            max: vec2(300.0, 300.0),
        })
        .add_systems(Startup, setup)
        .add_systems(Update, (sys_spawn_on_click, sys_plant_move, sys_plant_grow))
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
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
        if let Some(pos) = q_windows.single().cursor_position() {
            if let Ok((camera, gt)) = camera.get_single() {
                let pos = camera.viewport_to_world_2d(gt, pos).unwrap();
                if !bounds.in_bounds(pos) {
                    return;
                }
                commands.spawn(Plant::new_bundle(
                    asset_server.load("plant_base_test.png"),
                    asset_server.load("Crops/Carrot/carrot.png"),
                    pos,
                ));
            }
        }
    }
}

pub fn sys_plant_move(
    time: Res<Time>,
    bounds: Res<LevelBounds>,
    mut plants: Query<&mut Transform, With<Plant>>,
) {
    for mut plant_txfm in plants.iter_mut() {
        plant_txfm.translation -= Vec3::new(50.0 * time.delta_seconds(), 0.0, 0.0);
        if plant_txfm.translation.x <= bounds.min.x {
            plant_txfm.translation.x = bounds.max.x;
        }
    }
}

pub fn sys_plant_grow(
    time: Res<Time>,
    mut commands: Commands,
    mut plants: Query<(Entity, &Plant, &mut PlantGrowthState)>,
) {
    for (ent, plant, mut growth) in plants.iter_mut() {
        match *growth {
            PlantGrowthState::Empty {
                seconds_remaining: ref mut ticks_remaining,
            } => {
                *ticks_remaining -= time.delta_seconds();
                if *ticks_remaining <= 0.0 {
                    commands.entity(ent).insert(PlantGrowthState::Fruited);
                    commands
                        .spawn(SpriteBundle {
                            texture: plant.fruit_sprite.clone(),
                            transform: Transform::from_xyz(2.0, 2.0, 1.0),
                            ..Default::default()
                        })
                        .set_parent(ent);
                }
            }
            PlantGrowthState::Fruited => (),
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
pub struct Plant {
    pub fruit_sprite: Handle<Image>,
}

#[derive(Component)]
pub enum PlantGrowthState {
    Empty { seconds_remaining: f32 },
    Fruited,
}

impl Plant {
    fn new_bundle(texture: Handle<Image>, fruit_sprite: Handle<Image>, loc: Vec2) -> impl Bundle {
        (
            Plant { fruit_sprite },
            PlantGrowthState::Empty {
                seconds_remaining: 6.0,
            },
            SpriteBundle {
                texture,
                transform: Transform::from_xyz(loc.x, loc.y, 0.0),
                ..Default::default()
            },
        )
    }
}

#[derive(Component)]
pub struct Fruit;
