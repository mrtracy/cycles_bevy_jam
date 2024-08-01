use std::any::TypeId;

use bevy::{math::uvec2, prelude::*};
use bevy_mod_picking::{events::Pointer, prelude::On, selection::Select, PickableBundle};
use bevy_spatial::{kdtree::KDTree2, SpatialAccess};

use crate::normal_game::{
    fruit::{FruitGrowthState, HarvestFruitEvent},
    units::{Building, TowerRange},
};
use crate::{ui::CurrentIntention, AppState, GameType, SpatialTracked};

#[derive(Component)]
pub struct Harvester;

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
            Harvester,
            TowerRange(50),
            self.sprite_handle.clone(),
            Sprite {
                ..Default::default()
            },
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

    fn tile_size(&self) -> UVec2 {
        uvec2(1, 1)
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
                .run_if(in_state(AppState::Playing(GameType::NormalGame))),
        );
    }
}
