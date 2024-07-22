// Bevy code commonly triggers these lints and they may be important signals
// about code quality. They are sometimes hard to avoid though, and the CI
// workflow treats them as errors, so this allows them throughout the project.
// Feel free to delete this line.
#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use bevy::asset::AssetMetaCheck;
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
        .add_systems(Startup, setup)
        .add_systems(Update, (sys_spawn_on_click, sys_plant_move))
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
) {
    if buttons.just_pressed(MouseButton::Left) {
        if let Some(pos) = q_windows.single().cursor_position() {
            if let Ok((camera, gt)) = camera.get_single() {
                let pos = camera.viewport_to_world_2d(gt, pos).unwrap();
                commands.spawn(Plant::new_bundle(
                    asset_server.load("Crops/Carrot/carrot.png"),
                    pos,
                ));
            }
        }
    }
}

pub fn sys_plant_move(time: Res<Time>, mut plants: Query<&mut Transform, With<Plant>>) {
    for mut plant_txfm in plants.iter_mut() {
        plant_txfm.translation -= Vec3::new(50.0 * time.delta_seconds(), 0.0, 0.0);
        if plant_txfm.translation.x <= -100.0 {
            plant_txfm.translation.x = 100.0;
        }
    }
}

#[derive(Component)]
pub struct Plant;

impl Plant {
    fn new_bundle(texture: Handle<Image>, loc: Vec2) -> impl Bundle {
        (
            Plant,
            SpriteBundle {
                texture,
                transform: Transform::from_xyz(loc.x, loc.y, 0.0),
                ..Default::default()
            },
        )
    }
}
