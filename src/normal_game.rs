use std::{any::TypeId, time::Duration};

use crate::{
    construction_preview::BuildingPreviewPlugin,
    fruit,
    fruit_type::FruitSpeciesPlugin,
    level_map,
    nutrients::NutrientPlugin,
    ui::{self, CurrentIntention, OverlayMode},
    units::{
        BuildingTypeMap, BuildingTypePlugin, CurrentWave, DebugPlantType, IntermissionTimer,
        NextWaveQueue,
    },
    AppState, CameraPointerParam, GameType, MapQuery, MapQueryHelpers,
};
use bevy::prelude::*;
use bevy_mod_picking::{pointer::InputPress, prelude::PointerButton};

#[derive(SubStates, Default, Debug, Clone, PartialEq, Eq, Hash)]
#[source(AppState=AppState::Playing(GameType::NormalGame))]
pub enum PlayState {
    #[default]
    Setup,
    Intermission,
    Wave,
}

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
        app.add_systems(OnEnter(PlayState::Setup), level_map::kickoff_load)
            .add_sub_state::<PlayState>()
            .add_plugins(BuildingTypePlugin)
            .add_plugins(FruitSpeciesPlugin)
            .add_plugins(BuildingPreviewPlugin)
            .add_plugins(NutrientPlugin)
            .add_systems(OnEnter(PlayState::Setup), setup_game)
            .add_systems(
                Update,
                level_map::sys_wait_for_loading_level.run_if(in_state(PlayState::Setup)),
            )
            .add_systems(
                Update,
                (
                    sys_spawn_on_click,
                    fruit::sys_fruit_branch_spawn_fruit,
                    fruit::sys_fruit_grow,
                    ui::scoreboard,
                    ui::sys_ui_build_board,
                    ui::sys_selected_unit_ui.run_if(not(resource_equals(CurrentIntention::None))),
                    ui::sys_update_ui_title,
                    ui::sys_show_overlay,
                )
                    .run_if(in_state(PlayState::Wave).or_else(in_state(PlayState::Intermission))),
            )
            .insert_resource(OverlayMode::Normal)
            .observe(fruit::obs_fruit_harvested);
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
            map_pos += map_query.tile_center_to_corner();
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
