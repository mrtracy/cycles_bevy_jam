use std::any::TypeId;
use std::borrow::Cow;

use bevy::{prelude::*, utils::HashMap};

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

    let mut plant_type = Box::new(DebugPlantType::default());
    plant_type.init_assets(&asset_server);
    building_map
        .type_map
        .insert(TypeId::of::<DebugPlantType>(), plant_type);

    world.insert_resource(building_map);
}

pub struct BuildingTypePlugin;

impl Plugin for BuildingTypePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, sys_setup_building_types);
    }
}