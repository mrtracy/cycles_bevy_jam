use bevy::{
    app::{Plugin, Startup},
    asset::{AssetServer, Handle},
    prelude::{Res, ResMut, Resource},
    render::texture::Image,
};

#[derive(Debug, PartialEq, Hash)]
pub enum FruitGenus {
    Carrot,
}

pub struct FruitSpecies {
    #[allow(dead_code)]
    pub genus: FruitGenus,
    pub growth_time_secs: f32,
    pub fruit_image: Handle<Image>,
}

#[derive(Resource, Default)]
pub struct FruitSpeciesMap {
    pub species_vector: Vec<FruitSpecies>,
}

pub struct FruitSpeciesPlugin;

impl Plugin for FruitSpeciesPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<FruitSpeciesMap>()
            .add_systems(Startup, sys_init_fruit_species);
    }
}

pub fn sys_init_fruit_species(
    mut fruit_map: ResMut<FruitSpeciesMap>,
    asset_server: Res<AssetServer>,
) {
    fruit_map.species_vector.push(FruitSpecies {
        genus: FruitGenus::Carrot,
        growth_time_secs: 6.0,
        fruit_image: asset_server.load("Crops/Carrot/carrot.png"),
    });
}
