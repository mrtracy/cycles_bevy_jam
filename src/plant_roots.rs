use bevy::{math::vec2, prelude::*};

use crate::{fruit, fruit_type::FruitGenus, LevelBounds};

#[derive(Component)]
pub struct Plant {
    #[allow(dead_code)]
    pub genus: FruitGenus,
}

impl Plant {
    pub fn new_bundle(texture: Handle<Image>, loc: Vec2) -> impl Bundle {
        (
            Plant {
                genus: FruitGenus::Carrot,
            },
            SpriteBundle {
                texture,
                transform: Transform::from_xyz(loc.x, loc.y, 0.0),
                ..Default::default()
            },
        )
    }
}

pub fn sys_plant_move(
    time: Res<Time>,
    bounds: Res<LevelBounds>,
    mut plants: Query<&mut Transform, With<Plant>>,
) {
    for mut plant_txfm in plants.iter_mut() {
        plant_txfm.translation -= Vec3::new(50.0 * time.delta_seconds(), 0.0, 0.0);
        if plant_txfm.translation.x <= bounds.min.x {
            plant_txfm.translation.x = bounds.max.x;
        }
    }
}

pub fn sys_plant_fruit_spawn(
    mut commands: Commands,
    plants: Query<Entity, (With<Plant>, Without<PlantAttachedFruit>)>,
) {
    for plant_ent in plants.iter() {
        let fruit_id = commands
            .spawn(fruit::Fruit::new_bundle(0, vec2(1.0, 1.0)))
            .set_parent(plant_ent)
            .id();
        commands
            .entity(plant_ent)
            .insert(PlantAttachedFruit(fruit_id));
    }
}

#[derive(Component)]
#[allow(dead_code)]
pub struct PlantAttachedFruit(pub Entity);
