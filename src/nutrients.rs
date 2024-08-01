use bevy::prelude::*;

use crate::{level::TilePassable, normal_game::PlayState};

#[derive(Component)]
pub struct TileWater(pub u32);

pub struct NutrientPlugin;

pub fn sys_setup_nutrients(mut commands: Commands, tile_query: Query<(Entity, &TilePassable)>) {
    for (tile_ent, passable) in tile_query.iter() {
        commands
            .entity(tile_ent)
            .insert(TileWater(if passable.0 { 1000 } else { 0 }));
    }
}

impl Plugin for NutrientPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(PlayState::Setup), sys_setup_nutrients);
    }
}
