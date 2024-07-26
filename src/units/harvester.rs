use std::any::TypeId;

use bevy::{math::vec2, prelude::*};
use bevy_mod_picking::{
    events::Pointer,
    pointer::{InputPress, PointerButton},
    prelude::On,
    selection::Select,
    PickableBundle,
};
use bevy_spatial::{kdtree::KDTree2, SpatialAccess};

use crate::{
    fruit::{FruitGrowthState, HarvestFruitEvent},
    ui::CurrentIntention,
    units::Building,
    CameraPointerParam, GameState, LevelBounds, SpatialTracked,
};

use super::TowerRange;

#[derive(Component)]
pub struct Harvester {
    pub target: Vec2,
}

#[derive(Default)]
pub struct HarvesterType {
    sprite_handle: Handle<Image>,
}

impl Building for HarvesterType {
    fn init_assets(&mut self, asset_server: &AssetServer) {
        self.sprite_handle = asset_server.load("harvester_test.png");
    }

    fn name(&self) -> std::borrow::Cow<'static, str> {
        "Harvester".into()
    }

    fn construct_building(&self, commands: &mut Commands, target: Entity) {
        commands.entity(target).insert((
            Harvester {
                target: vec2(0., 0.),
            },
            TowerRange(50),
            self.sprite_handle.clone(),
            Sprite::default(),
            PickableBundle::default(),
            On::<Pointer<Select>>::commands_mut(|event, commands| {
                commands.insert_resource(CurrentIntention::Command(
                    TypeId::of::<HarvesterType>(),
                    event.target,
                ));
            }),
        ));
    }

    fn sprite_image(&self) -> &Handle<Image> {
        &self.sprite_handle
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

pub fn sys_harvester_look_for_fruit(
    mut commands: Commands,
    spatial_tree: Res<KDTree2<SpatialTracked>>,
    harvesters: Query<(Entity, &Harvester, &GlobalTransform, &TowerRange)>,
    fruit: Query<&FruitGrowthState>,
) {
    for (harvester_ent, _harvester, transform, range) in harvesters.iter() {
        for (_, entity) in
            spatial_tree.within_distance(transform.translation().xy(), range.0 as f32)
        {
            let Some(entity) = entity else { continue };
            let Ok(FruitGrowthState::Fruited) = fruit.get(entity) else {
                continue;
            };
            commands.trigger_targets(HarvestFruitEvent { harvester_ent }, entity);
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

pub struct HarvesterPlugin;

impl Plugin for HarvesterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                sys_harvester_look_for_fruit,
                // sys_harvester_target_set,
                // sys_harvester_move_to_target,
            )
                .run_if(in_state(GameState::Playing)),
        );
    }
}
