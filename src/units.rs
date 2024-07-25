use std::any::TypeId;
use std::borrow::Cow;

use bevy::{prelude::*, utils::HashMap};
use harvester::{HarvesterPlugin, HarvesterType};

use crate::{
    fruit::{FruitBranch, FruitBranchBundle},
    plant_roots,
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
            .insert(plant_roots::Plant::new_bundle(
                self.sprite_image_handle.clone(),
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
            .add_plugins(HarvesterPlugin);
    }
}

pub mod harvester;
