use std::any::TypeId;
use std::borrow::Cow;

use bevy::{ecs::system::SystemId, prelude::*, utils::HashMap};

use crate::{
    fruit::{FruitBranch, FruitBranchBundle},
    plant_roots, Harvester,
};

#[derive(Component)]
pub struct SpriteData {
    pub primary_sprite: Handle<Image>,
}

#[derive(Component)]
pub struct BuildingType {
    pub name: Cow<'static, str>,
    pub constructor_system_id: SystemId<Entity, ()>,
}

#[derive(Resource, Default)]
pub struct BuildingTypeMap {
    pub type_map: HashMap<TypeId, Entity>,
}

pub struct HarvesterType;

impl HarvesterType {
    pub const SPRITE_PATH: &'static str = "harvester_test.png";
    pub const NAME: &'static str = "Harvester";
    pub fn constructor(
        In(target): In<Entity>,
        mut commands: Commands,
        asset_server: Res<AssetServer>,
    ) {
        commands
            .entity(target)
            .insert(Harvester::new_bundle(&asset_server, 50));
    }
}

pub struct DebugPlantType;

impl DebugPlantType {
    pub const SPRITE_PATH: &'static str = "plant_base_test.png";
    pub const NAME: &'static str = "Debug Roots";
    pub fn constructor(
        In(target): In<Entity>,
        mut commands: Commands,
        asset_server: Res<AssetServer>,
    ) {
        commands
            .entity(target)
            .insert(plant_roots::Plant::new_bundle(
                asset_server.load("plant_base_test.png"),
            ))
            .with_children(|child_commands| {
                child_commands.spawn(FruitBranchBundle {
                    branch: FruitBranch { species: 0 },
                    sprite: SpriteBundle {
                        ..Default::default()
                    },
                });
            });
    }
}

pub fn sys_setup_building_types(world: &mut World) {
    let mut building_map = BuildingTypeMap::default();
    let asset_server = world.get_resource::<AssetServer>().unwrap().clone();
    macro_rules! register_type {
        ($desc:ident) => {
            let constructor_system_id = world.register_system($desc::constructor);
            let build_type_id = world
                .spawn((
                    BuildingType {
                        name: <$desc>::NAME.into(),
                        constructor_system_id,
                    },
                    SpriteData {
                        primary_sprite: asset_server.load($desc::SPRITE_PATH),
                    },
                ))
                .id();
            building_map
                .type_map
                .insert(TypeId::of::<$desc>(), build_type_id);
        };
    }

    register_type!(HarvesterType);
    register_type!(DebugPlantType);
    world.insert_resource(building_map);
}

pub struct BuildingTypePlugin;

impl Plugin for BuildingTypePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, sys_setup_building_types);
    }
}
