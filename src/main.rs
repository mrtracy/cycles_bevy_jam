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
use bevy_egui::EguiPlugin;
use bevy_mod_picking::events::Pointer;
use bevy_mod_picking::pointer::{InputPress, PointerButton, PointerId, PointerLocation};
use bevy_mod_picking::prelude::On;
use bevy_mod_picking::selection::{Select, SelectionPluginSettings};
use bevy_mod_picking::{DefaultPickingPlugins, PickableBundle};
use bevy_spatial::kdtree::KDTree2;
use bevy_spatial::{AutomaticUpdate, SpatialAccess, SpatialStructure, TransformMode};
use buildings::BuildingTypePlugin;
use construction_preview::BuildingPreviewPlugin;
use fruit_type::FruitSpeciesPlugin;
use ui::CurrentIntention;

mod construction_preview;
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
        .add_plugins(DefaultPickingPlugins)
        .insert_resource(SelectionPluginSettings {
            use_multiselect_default_inputs: false,
            ..Default::default()
        })
        .add_plugins(EguiPlugin)
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
                    sys_harvester_look_for_fruit,
                    // sys_harvester_target_set,
                    // sys_harvester_move_to_target,
                    fruit::sys_fruit_branch_spawn_fruit,
                    fruit::sys_fruit_grow,
                    ui::scoreboard,
                    ui::sys_selected_unit_ui
                        .run_if(not(resource_equals(CurrentIntention::None))),
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
    building_data: Query<(&buildings::SpriteData, &buildings::BuildingType)>,
) {
    for press in press_events
        .read()
        .filter(|p| p.is_just_down(PointerButton::Primary))
    {
        let Some(pos) = pointers.get_world_pointer_location(press.pointer_id) else {
            continue;
        };
        match *current_inspector {
            CurrentIntention::Prospective(ref building) => {
                info!("Prospective building click");
                if bounds.in_bounds(pos) {
                    let Ok((_, building_type)) = building_data.get(*building) else {
                        info!("Propective building type was not found");
                        continue;
                    };
                    let new_entity = commands
                        .spawn(SpatialBundle {
                            transform: Transform::from_xyz(pos.x, pos.y, 0.),
                            ..Default::default()
                        })
                        .id();

                    commands.run_system_with_input(building_type.constructor_system_id, new_entity);
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
            asset_server.load::<Image>("harvester_test.png"),
            Sprite::default(),
            PickableBundle::default(),
            On::<Pointer<Select>>::commands_mut(|event, commands| {
                commands.insert_resource(CurrentIntention::Harvester(event.target));
            }),
        )
    }
}

pub fn sys_harvester_target_set(
    mut press_events: EventReader<InputPress>,
    pointers: CameraPointerParam,
    mut harvester: Query<(&Transform, &mut Harvester)>,
    bounds: Res<LevelBounds>,
    mut gizmos: Gizmos,
) {
    for press in press_events
        .read()
        .filter(|p| p.is_just_down(PointerButton::Secondary))
    {
        let Some(pos) = pointers.get_world_pointer_location(press.pointer_id) else {
            continue;
        };
        if bounds.in_bounds(pos) {
            harvester
                .get_single_mut()
                .map(|(_, mut h)| h.as_mut().target = vec2(pos.x, pos.y))
                .expect("Failed to update target");
            gizmos.arrow_2d(harvester.single().0.translation.xy(), pos, Color::WHITE);
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

pub mod buildings;
