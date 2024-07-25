use std::borrow::Cow;
use std::{any::TypeId, time::Duration};

use bevy::{prelude::*, utils::HashMap};
use bevy_ecs_tilemap::map::{TilemapGridSize, TilemapType};
use harvester::{HarvesterPlugin, HarvesterType};

use crate::PlayState;
use crate::{
    fruit::{FruitBranch, FruitBranchBundle},
    level::{CurrentLevel, TilePath},
    tree, GameState,
};

pub trait Building: Send + Sync {
    fn init_assets(&mut self, asset_server: &AssetServer);

    fn name(&self) -> Cow<'static, str>;

    fn construct_building(&self, commands: &mut Commands, target: Entity);

    fn sprite_image(&self) -> &Handle<Image>;
}

#[derive(Resource, Default)]
pub struct BuildingTypeMap {
    pub type_map: HashMap<TypeId, Box<dyn Building>>,
}

#[derive(Default)]
pub struct DebugPlantType {
    pub sprite_image_handle: Handle<Image>,
}

impl DebugPlantType {
    pub const NAME: &'static str = "Debug Roots";
}

impl Building for DebugPlantType {
    fn init_assets(&mut self, asset_server: &AssetServer) {
        self.sprite_image_handle = asset_server.load("plant_base_test.png")
    }

    fn name(&self) -> Cow<'static, str> {
        Self::NAME.into()
    }

    fn construct_building(&self, commands: &mut Commands, target: Entity) {
        commands
            .entity(target)
            .insert(tree::Tree::new_bundle(self.sprite_image_handle.clone()))
            .with_children(|child_commands| {
                child_commands.spawn(FruitBranchBundle {
                    branch: FruitBranch { species: 0 },
                    sprite: SpriteBundle {
                        ..Default::default()
                    },
                });
            });
    }

    fn sprite_image(&self) -> &Handle<Image> {
        &self.sprite_image_handle
    }
}

pub fn sys_setup_building_types(world: &mut World) {
    let mut building_map = BuildingTypeMap::default();
    let asset_server = world.get_resource::<AssetServer>().unwrap().clone();

    macro_rules! register_type {
        ($building_type:ident) => {{
            let mut building_type = Box::new($building_type::default());
            building_type.init_assets(&asset_server);
            building_map
                .type_map
                .insert(TypeId::of::<$building_type>(), building_type);
        }};
    }

    register_type!(DebugPlantType);
    register_type!(HarvesterType);

    world.insert_resource(building_map);
}

pub struct BuildingTypePlugin;

impl Plugin for BuildingTypePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, sys_setup_building_types)
            .add_systems(
                Update,
                sys_follow_tile_path.run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                (
                    sys_entity_leaves_wave,
                    sys_wave_place_entities,
                    sys_detect_wave_completed,
                )
                    .run_if(in_state(PlayState::Wave)),
            )
            .add_systems(
                Update,
                (sys_intermission_timer).run_if(in_state(PlayState::Intermission)),
            )
            .add_plugins(HarvesterPlugin);
    }
}

#[derive(Component)]
pub struct PathFollower {
    pub current_dist: f32,
    pub speed: f32,
}

#[derive(Component)]
pub struct PathCompleted;

pub fn sys_follow_tile_path(
    mut commands: Commands,
    time: Res<Time>,
    path_q: Query<
        (&TilePath, &TilemapGridSize, &TilemapType, &GlobalTransform),
        With<CurrentLevel>,
    >,
    mut followers: Query<(Entity, &mut Transform, &mut PathFollower), Without<PathCompleted>>,
) {
    let Ok((TilePath { path }, grid_size, map_type, map_transform)) = path_q.get_single() else {
        return;
    };
    for (follower_ent, mut follower_tfm, mut follower) in followers.iter_mut() {
        follower.current_dist += follower.speed * time.delta_seconds();
        let target_idx = follower.current_dist.floor() as usize;
        if target_idx >= path.len() - 1 {
            commands.entity(follower_ent).insert(PathCompleted);
            follower_tfm.translation = (
                path[path.len() - 1].center_in_world(grid_size, map_type),
                5.,
            )
                .into()
        } else {
            let current_idx_pos = path[target_idx].center_in_world(grid_size, map_type);
            let next_idx_pos = path[target_idx + 1].center_in_world(grid_size, map_type);
            follower_tfm.translation = map_transform.translation()
                + Into::<Vec3>::into((
                    current_idx_pos.lerp(next_idx_pos, follower.current_dist.fract()),
                    5.,
                ));
        }
    }
}

#[derive(Component)]
pub struct QueuedForNextWave;

#[derive(Resource, Default)]
pub struct NextWaveQueue(pub Vec<Entity>);

#[derive(Resource)]
pub struct IntermissionTimer(pub Timer);

#[derive(Resource, Default)]
pub struct CurrentWave {
    pub unit_queue: Vec<Entity>,
    pub next_unit_timer: Timer,
}

impl CurrentWave {
    pub fn new(unit_spacing_time: Duration) -> Self {
        CurrentWave {
            next_unit_timer: Timer::new(unit_spacing_time, TimerMode::Once),
            ..Default::default()
        }
    }
}

pub fn sys_intermission_timer(
    time: Res<Time>,
    mut intermission_timer: ResMut<IntermissionTimer>,
    mut next_wave: ResMut<NextWaveQueue>,
    mut current_wave: ResMut<CurrentWave>,
    mut next_play_state: ResMut<NextState<PlayState>>,
) {
    intermission_timer.0.tick(time.delta());
    if !intermission_timer.0.finished() {
        return;
    };

    current_wave.unit_queue.extend(next_wave.0.drain(..).rev());
    next_play_state.set(PlayState::Wave);
}

pub fn sys_entity_leaves_wave(
    mut commands: Commands,
    mut next_wave: ResMut<NextWaveQueue>,
    leave_query: Query<Entity, With<PathCompleted>>,
) {
    for left in leave_query.iter() {
        next_wave.0.push(left);
        commands
            .entity(left)
            .remove::<(PathCompleted, PathFollower)>()
            .insert((
                Visibility::Hidden,
                Transform::from_translation(Vec3::new(-10000.0, 0.0, -10000.0)),
            ));
    }
}

pub fn sys_detect_wave_completed(
    current_wave: Res<CurrentWave>,
    unit_query: Query<Entity, With<PathFollower>>,
    mut next_play_state: ResMut<NextState<PlayState>>,
    mut intermission_timer: ResMut<IntermissionTimer>,
) {
    if current_wave.unit_queue.is_empty() && unit_query.is_empty() {
        intermission_timer.0.reset();
        next_play_state.set(PlayState::Intermission);
    }
}

pub fn sys_wave_place_entities(
    mut commands: Commands,
    mut wave: ResMut<CurrentWave>,
    time: Res<Time>,
) {
    if wave.unit_queue.is_empty() {
        return;
    };
    wave.next_unit_timer.tick(time.delta());
    if wave.next_unit_timer.finished() {
        wave.next_unit_timer.reset();
        let next_unit = wave.unit_queue.pop().unwrap();
        commands.entity(next_unit).insert((
            Visibility::Inherited,
            PathFollower {
                current_dist: 0.0,
                speed: 20.0,
            },
        ));
    }
}

pub mod harvester;
