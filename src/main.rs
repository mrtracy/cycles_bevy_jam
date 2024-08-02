// Bevy code commonly triggers these lints and they may be important signals
// about code quality. They are sometimes hard to avoid though, and the CI
// workflow treats them as errors, so this allows them throughout the project.
// Feel free to delete this line.
#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use std::time::Duration;

use bevy::asset::AssetMetaCheck;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_ecs_tilemap::map::{TilemapGridSize, TilemapSize, TilemapType};
use bevy_ecs_tilemap::tiles::TilePos;
use bevy_ecs_tilemap::TilemapPlugin;
use bevy_egui::EguiPlugin;
use bevy_mod_picking::pointer::{PointerId, PointerLocation};
use bevy_mod_picking::selection::SelectionPluginSettings;
use bevy_mod_picking::DefaultPickingPlugins;
use bevy_pancam::{PanCam, PanCamPlugin};
use bevy_rapier2d::plugin::{NoUserData, RapierPhysicsPlugin};
use bevy_rapier2d::render::RapierDebugRenderPlugin;
use bevy_spatial::{AutomaticUpdate, SpatialStructure, TransformMode};
use fruit_game::NormalGamePlugin;

mod fruit_game;
mod ui;
mod voting;

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameType {
    #[default]
    NormalGame,
    TimsGame,
}

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum AppState {
    Init,
    #[default]
    MainMenu,
    Playing(GameType),
}

#[derive(SubStates, Default, Debug, Clone, PartialEq, Eq, Hash)]
#[source(AppState=AppState::Playing(GameType::TimsGame))]
pub enum TimsGameState {
    #[default]
    Setup,
    Playing,
    Paused,
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
                .with_frequency(Duration::from_secs_f32(0.2))
                .with_spatial_ds(SpatialStructure::KDTree2)
                .with_transform(TransformMode::GlobalTransform),
        )
        .add_plugins(DefaultPickingPlugins)
        .insert_resource(SelectionPluginSettings {
            use_multiselect_default_inputs: false,
            ..Default::default()
        })
        .add_plugins(EguiPlugin)
        .add_plugins(TilemapPlugin)
        .add_plugins(PanCamPlugin)
        .add_plugins(ui::UiPlugin)
        .add_plugins(voting::VotingPlugin)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugins(RapierDebugRenderPlugin::default())
        .insert_resource(Score(0))
        .insert_resource(Level::default())
        .init_state::<AppState>()
        .add_sub_state::<TimsGameState>()
        .add_systems(Startup, (setup, ui::sys_setup_ui_nodes))
        .add_systems(
            Update,
            ((ui::main_menu).run_if(in_state(AppState::MainMenu)),),
        )
        .add_plugins(NormalGamePlugin)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle::default(),
        PanCam {
            grab_buttons: vec![MouseButton::Middle],
            enabled: true,
            ..Default::default()
        },
    ));
}

#[derive(SystemParam)]
pub struct CameraPointerParam<'w, 's> {
    pub pointers: Query<'w, 's, (&'static PointerId, &'static PointerLocation)>,
    pub cameras: Query<'w, 's, (&'static Camera, &'static GlobalTransform)>,
}

impl<'w, 's> CameraPointerParam<'w, 's> {
    pub fn get_world_pointer_location(&self, pointer_id: PointerId) -> Option<Vec2> {
        let Some((
            _,
            PointerLocation {
                location: Some(loc),
            },
        )) = self.pointers.iter().find(|(pid, _)| **pid == pointer_id)
        else {
            return None;
        };
        let Ok((camera, gt)) = self.cameras.get_single() else {
            return None;
        };
        camera.viewport_to_world_2d(gt, loc.position)
    }
}

pub type MapQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static TilemapType,
        &'static TilemapSize,
        &'static TilemapGridSize,
        &'static GlobalTransform,
    ),
>;

pub trait MapQueryHelpers {
    fn snap_to_tile_center(&self, pos: &Vec2) -> Option<Vec3>;
    fn tile_center_to_corner(&self) -> Vec3;
}

impl<'w, 's> MapQueryHelpers for MapQuery<'w, 's> {
    fn snap_to_tile_center(&self, pos: &Vec2) -> Option<Vec3> {
        let Some((map_type, map_size, map_grid_size, map_transform)) = self.get_single().ok()
        else {
            warn!("Map data not available for placing buildings");
            return None;
        };
        let clicked_tile = TilePos::from_world_pos(
            &(*pos - map_transform.translation().xy()),
            map_size,
            map_grid_size,
            map_type,
        )?;
        Some(
            map_transform.translation()
                + Vec3::from((clicked_tile.center_in_world(map_grid_size, map_type), 5.0)),
        )
    }

    fn tile_center_to_corner(&self) -> Vec3 {
        let Some((_, _, map_grid_size, _)) = self.get_single().ok() else {
            warn!("Map data not available for placing buildings");
            return Vec3::ZERO;
        };
        Vec3::new(map_grid_size.x, -map_grid_size.y, 0.0) / 2.0
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct Score(usize);

#[derive(Resource, Default, Deref, DerefMut)]
pub struct Level {
    level: usize,
}
