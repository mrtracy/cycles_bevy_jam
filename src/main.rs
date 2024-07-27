// Bevy code commonly triggers these lints and they may be important signals
// about code quality. They are sometimes hard to avoid though, and the CI
// workflow treats them as errors, so this allows them throughout the project.
// Feel free to delete this line.
#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use std::any::TypeId;
use std::time::Duration;

use bevy::asset::AssetMetaCheck;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_ecs_tilemap::map::{TilemapGridSize, TilemapSize, TilemapType};
use bevy_ecs_tilemap::tiles::TilePos;
use bevy_ecs_tilemap::TilemapPlugin;
use bevy_egui::EguiPlugin;
use bevy_mod_picking::pointer::{InputPress, PointerButton, PointerId, PointerLocation};
use bevy_mod_picking::selection::SelectionPluginSettings;
use bevy_mod_picking::DefaultPickingPlugins;
use bevy_pancam::{PanCam, PanCamPlugin};
use bevy_spatial::{AutomaticUpdate, SpatialStructure, TransformMode};
use construction_preview::BuildingPreviewPlugin;
use fruit_type::FruitSpeciesPlugin;
use ui::CurrentIntention;
use units::{
    BuildingTypeMap, BuildingTypePlugin, CurrentWave, DebugPlantType, IntermissionTimer,
    NextWaveQueue,
};

mod construction_preview;
mod fruit;
mod fruit_type;
mod level;
mod tree;
mod ui;
mod units;

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
    #[default]
    MainMenu,
    Loading,
    Playing,
    GameOver,
}

#[derive(SubStates, Default, Debug, Clone, PartialEq, Eq, Hash)]
#[source(GameState=GameState::Playing)]
pub enum PlayState {
    #[default]
    Setup,
    Intermission,
    Wave,
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
        .add_plugins(BuildingTypePlugin)
        .add_plugins(FruitSpeciesPlugin)
        .add_plugins(BuildingPreviewPlugin)
        .insert_resource(Score(0))
        .insert_resource(CurrentIntention::None)
        .insert_resource(NextWaveQueue::default())
        .init_state::<GameState>()
        .add_sub_state::<PlayState>()
        .add_systems(Startup, (setup, ui::sys_setup_ui_nodes))
        .add_systems(
            Update,
            (
                (ui::main_menu).run_if(in_state(GameState::MainMenu)),
                (
                    sys_spawn_on_click,
                    fruit::sys_fruit_branch_spawn_fruit,
                    fruit::sys_fruit_grow,
                    ui::scoreboard,
                    ui::sys_ui_build_board,
                    ui::sys_selected_unit_ui.run_if(not(resource_equals(CurrentIntention::None))),
                    ui::sys_update_ui_title,
                )
                    .run_if(in_state(GameState::Playing)),
            ),
        )
        .add_systems(OnEnter(GameState::Loading), level::kickoff_load)
        .add_systems(OnEnter(PlayState::Setup), setup_game)
        .add_systems(
            Update,
            level::sys_wait_for_loading_level.run_if(in_state(GameState::Loading)),
        )
        .observe(fruit::obs_fruit_harvested)
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

fn setup_game(
    mut commands: Commands,
    buildings: Res<BuildingTypeMap>,
    mut next_play_state: ResMut<NextState<PlayState>>,
) {
    let tree_type = buildings
        .type_map
        .get(&TypeId::of::<DebugPlantType>())
        .unwrap();
    let mut initial_unit_queue = vec![];
    for _ in 0..10 {
        let target = commands
            .spawn(SpatialBundle {
                transform: Transform::from_xyz(-10000.0, 0., 0.),
                visibility: Visibility::Hidden,
                ..Default::default()
            })
            .id();
        tree_type.construct_building(&mut commands, target);
        commands.entity(target).insert(Visibility::Hidden);
        initial_unit_queue.push(target);
    }
    commands.insert_resource(NextWaveQueue(initial_unit_queue));
    commands.insert_resource(CurrentWave::new(Duration::from_secs(1)));
    commands.insert_resource(IntermissionTimer(Timer::new(
        Duration::from_secs(3),
        TimerMode::Once,
    )));

    info!("Setup game complete!");
    next_play_state.set(PlayState::Intermission);
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

pub fn sys_spawn_on_click(
    mut commands: Commands,
    mut press_events: EventReader<InputPress>,
    pointers: CameraPointerParam,
    current_inspector: Res<CurrentIntention>,
    map_query: MapQuery,
    building_types: Res<BuildingTypeMap>,
) {
    for press in press_events
        .read()
        .filter(|p| p.is_just_down(PointerButton::Primary))
    {
        let Some(pos) = pointers.get_world_pointer_location(press.pointer_id) else {
            continue;
        };
        if let CurrentIntention::Prospective(ref building_type_id) = *current_inspector {
            let Some(building_type) = building_types.type_map.get(building_type_id) else {
                info!("Propective building type was not found");
                continue;
            };
            let Some(mut map_pos) = map_query.snap_to_tile_center(&pos) else {
                continue;
            };
            map_pos -= map_query.tile_center_to_corner();
            let new_entity = commands
                .spawn(SpatialBundle {
                    transform: Transform::from_translation(map_pos),
                    ..Default::default()
                })
                .id();
            building_type.construct_building(&mut commands, new_entity);
            commands.insert_resource(CurrentIntention::None);
        }
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct Score(usize);
