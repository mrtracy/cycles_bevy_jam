// Bevy code commonly triggers these lints and they may be important signals
// about code quality. They are sometimes hard to avoid though, and the CI
// workflow treats them as errors, so this allows them throughout the project.
// Feel free to delete this line.
#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use std::time::Duration;

use bevy::asset::AssetMetaCheck;
use bevy::ecs::system::SystemParam;
use bevy::math::vec2;
use bevy::prelude::*;
use bevy_ecs_tilemap::TilemapPlugin;
use bevy_egui::EguiPlugin;
use bevy_mod_picking::pointer::{InputPress, PointerButton, PointerId, PointerLocation};
use bevy_mod_picking::selection::SelectionPluginSettings;
use bevy_mod_picking::DefaultPickingPlugins;
use bevy_spatial::{AutomaticUpdate, SpatialStructure, TransformMode};
use construction_preview::BuildingPreviewPlugin;
use fruit_type::FruitSpeciesPlugin;
use ui::CurrentIntention;
use units::{BuildingTypeMap, BuildingTypePlugin};

mod construction_preview;
mod fruit;
mod fruit_type;
mod plant_roots;
mod ui;
pub mod units;

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
    #[default]
    MainMenu,
    Loading,
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
        .add_plugins(DefaultPickingPlugins)
        .insert_resource(SelectionPluginSettings {
            use_multiselect_default_inputs: false,
            ..Default::default()
        })
        .add_plugins(EguiPlugin)
        .add_plugins(TilemapPlugin)
        .add_plugins(BuildingTypePlugin)
        .add_plugins(FruitSpeciesPlugin)
        .add_plugins(BuildingPreviewPlugin)
        .insert_resource(LevelBounds {
            min: vec2(-300.0, -300.0),
            max: vec2(300.0, 300.0),
        })
        .insert_resource(Score(0))
        .insert_resource(CurrentIntention::None)
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
                    fruit::sys_fruit_branch_spawn_fruit,
                    fruit::sys_fruit_grow,
                    ui::scoreboard,
                    ui::sys_selected_unit_ui.run_if(not(resource_equals(CurrentIntention::None))),
                )
                    .run_if(in_state(GameState::Playing)),
            ),
        )
        .add_systems(OnEnter(GameState::Loading), kickoff_load)
        .add_systems(
            Update,
            sys_wait_for_loading_level.run_if(in_state(GameState::Loading)),
        )
        .observe(fruit::obs_fruit_harvested)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

#[derive(Resource)]
pub struct LoadingLevel(pub Handle<Image>);

fn kickoff_load(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(LoadingLevel(asset_server.load("levels/level1.png")));
}

fn sys_wait_for_loading_level(
    mut commands: Commands,
    loading_level: Res<LoadingLevel>,
    images: Res<Assets<Image>>,
    asset_server: Res<AssetServer>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    use bevy_ecs_tilemap::prelude::*;

    let level_data_handle = &loading_level.0;

    let Some(image_data) = images.get(level_data_handle) else {
        return;
    };

    let texture_handle: Handle<Image> = asset_server.load("tiles.png");
    let map_size = TilemapSize {
        x: image_data.width(),
        y: image_data.height(),
    };

    let tilemap_entity = commands.spawn_empty().id();

    let mut tile_storage = TileStorage::empty(map_size);

    let grayscale = image_data.clone().try_into_dynamic().unwrap().to_luma8();

    for x in 0..map_size.x {
        for y in 0..map_size.y {
            let tile_pos = TilePos { x, y };
            let tile_entity = commands
                .spawn(TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    texture_index: match grayscale.get_pixel(x, y).0[0] {
                        x if x < 50 => TileTextureIndex(0),
                        _ => TileTextureIndex(1),
                    },
                    ..Default::default()
                })
                .id();
            tile_storage.set(&tile_pos, tile_entity);
        }
    }

    let tile_size = TilemapTileSize { x: 16.0, y: 16.0 };
    let grid_size = tile_size.into();
    let map_type = TilemapType::default();

    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size,
        map_type,
        size: map_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(texture_handle),
        tile_size,
        transform: get_tilemap_center_transform(&map_size, &grid_size, &map_type, 0.0),
        ..Default::default()
    });

    commands.remove_resource::<LoadingLevel>();
    next_game_state.set(GameState::Playing);
}

pub fn sys_draw_border(mut gizmos: Gizmos, bounds: Res<LevelBounds>) {
    gizmos.line_2d(bounds.min, bounds.max.with_y(bounds.min.y), Color::WHITE);
    gizmos.line_2d(bounds.min, bounds.max.with_x(bounds.min.x), Color::WHITE);
    gizmos.line_2d(bounds.max, bounds.max.with_y(bounds.min.y), Color::WHITE);
    gizmos.line_2d(bounds.max, bounds.max.with_x(bounds.min.x), Color::WHITE);
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

pub fn sys_spawn_on_click(
    mut commands: Commands,
    mut press_events: EventReader<InputPress>,
    pointers: CameraPointerParam,
    current_inspector: Res<CurrentIntention>,
    bounds: Res<LevelBounds>,
    building_types: Res<BuildingTypeMap>,
) {
    for press in press_events
        .read()
        .filter(|p| p.is_just_down(PointerButton::Primary))
    {
        let Some(pos) = pointers.get_world_pointer_location(press.pointer_id) else {
            continue;
        };
        match *current_inspector {
            CurrentIntention::Prospective(ref building_type_id) => {
                info!("Prospective building click");
                if bounds.in_bounds(pos) {
                    let Some(building_type) = building_types.type_map.get(building_type_id) else {
                        info!("Propective building type was not found");
                        continue;
                    };
                    let new_entity = commands
                        .spawn(SpatialBundle {
                            transform: Transform::from_xyz(pos.x, pos.y, 0.),
                            ..Default::default()
                        })
                        .id();
                    building_type.construct_building(&mut commands, new_entity);
                    commands.insert_resource(CurrentIntention::None);
                }
            }
            _ => {}
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

#[derive(Resource, Deref, DerefMut)]
pub struct Score(usize);
