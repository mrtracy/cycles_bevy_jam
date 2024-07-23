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

mod ui;

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
        .insert_resource(LevelBounds {
            min: vec2(-300.0, -300.0),
            max: vec2(300.0, 300.0),
        })
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                ui::example,
                sys_spawn_on_click,
                sys_plant_move,
                sys_harvester_look_for_fruit,
                sys_harvester_target_set,
                sys_harvester_move_to_target,
                sys_fruit_spawn,
                sys_fruit_grow,
            ),
        )
        .observe(obs_fruit_harvested)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn(Harvester::new_bundle(&asset_server, 50));
}

pub fn sys_harvester_look_for_fruit(
    mut commands: Commands,
    spatial_tree: Res<KDTree2<SpatialTracked>>,
    harvesters: Query<(Entity, &Harvester, &Transform)>,
    fruit: Query<&FruitGrowthState>,
) {
    for (harvester_ent, harvester, transform) in harvesters.iter() {
        for (_, entity) in spatial_tree.within_distance(
            transform.translation.xy() + vec2(16.0, 16.0),
            harvester.range_units as f32,
        ) {
            let Some(entity) = entity else { continue };
            let Ok(FruitGrowthState::Fruited) = fruit.get(entity) else {
                continue;
            };
            commands.trigger_targets(HarvestFruit { harvester_ent }, entity);
        }
    }
}

#[derive(Event)]
pub struct HarvestFruit {
    pub harvester_ent: Entity,
}

pub fn obs_fruit_harvested(event: Trigger<HarvestFruit>, mut commands: Commands) {
    info!("Triggered fruit harvest");

    commands.entity(event.entity()).insert((
        FruitGrowthState::Empty {
            seconds_remaining: 6.0,
        },
        Visibility::Hidden,
    ));
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

pub fn sys_fruit_spawn(
    mut commands: Commands,
    plants: Query<(Entity, &Plant), Without<PlantAttachedFruit>>,
) {
    for (plant_ent, plant) in plants.iter() {
        let fruit_id = commands
            .spawn(Fruit::new_bundle(
                plant.fruit_sprite.clone(),
                vec2(1.0, 1.0),
            ))
            .set_parent(plant_ent)
            .id();
        commands
            .entity(plant_ent)
            .insert(PlantAttachedFruit(fruit_id));
    }
}

pub fn sys_fruit_grow(
    time: Res<Time>,
    mut commands: Commands,
    mut fruits: Query<(Entity, &mut FruitGrowthState)>,
) {
    for (fruit_ent, mut growth) in fruits.iter_mut() {
        match *growth {
            FruitGrowthState::Empty {
                seconds_remaining: ref mut ticks_remaining,
            } => {
                *ticks_remaining -= time.delta_seconds();
                if *ticks_remaining <= 0.0 {
                    commands
                        .entity(fruit_ent)
                        .insert((FruitGrowthState::Fruited, Visibility::Inherited));
                }
            }
            FruitGrowthState::Fruited => (),
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
pub enum FruitGrowthState {
    Empty { seconds_remaining: f32 },
    Fruited,
}

impl Plant {
    fn new_bundle(texture: Handle<Image>, fruit_sprite: Handle<Image>, loc: Vec2) -> impl Bundle {
        (
            Plant { fruit_sprite },
            SpriteBundle {
                texture,
                transform: Transform::from_xyz(loc.x, loc.y, 0.0),
                ..Default::default()
            },
        )
    }
}

#[derive(Component)]
pub struct PlantAttachedFruit(pub Entity);

#[derive(Component)]
pub struct Fruit;

impl Fruit {
    fn new_bundle(texture: Handle<Image>, loc: Vec2) -> impl Bundle {
        (
            SpatialTracked,
            SpriteBundle {
                texture,
                transform: Transform::from_xyz(loc.x, loc.y, 0.0),
                visibility: Visibility::Hidden,
                ..Default::default()
            },
            FruitGrowthState::Empty {
                seconds_remaining: 6.0,
            },
        )
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
