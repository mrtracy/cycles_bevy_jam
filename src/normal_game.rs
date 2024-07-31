use std::{any::TypeId, time::Duration};

use crate::{
    level,
    units::{BuildingTypeMap, CurrentWave, DebugPlantType, IntermissionTimer, NextWaveQueue},
    PlayState,
};
use bevy::prelude::*;

pub(crate) fn setup_game(mut commands: Commands, buildings: Res<BuildingTypeMap>) {
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
}

pub struct NormalGamePlugin;

impl Plugin for NormalGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(PlayState::Setup), level::kickoff_load)
            .add_systems(OnEnter(PlayState::Setup), setup_game)
            .add_systems(
                Update,
                level::sys_wait_for_loading_level.run_if(in_state(PlayState::Setup)),
            );
    }
}
