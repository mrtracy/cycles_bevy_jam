use bevy::prelude::*;

use crate::{fruit_type::FruitSpeciesMap, SpatialTracked};

#[derive(Event)]
pub struct HarvestFruitEvent {
    #[allow(dead_code)]
    pub harvester_ent: Entity,
}

#[derive(Component)]
pub struct Fruit {
    fruit_type: usize,
}

#[derive(Component)]
pub enum FruitGrowthState {
    Empty { seconds_of_growth: f32 },
    Fruited,
}

impl Fruit {
    pub fn new_bundle(fruit_type: usize, loc: Vec2) -> impl Bundle {
        (
            SpatialTracked,
            SpriteBundle {
                transform: Transform::from_xyz(loc.x, loc.y, 0.0),
                ..Default::default()
            },
            Fruit { fruit_type },
            FruitGrowthState::Empty {
                seconds_of_growth: 0.0,
            },
        )
    }
}

pub fn sys_fruit_grow(
    time: Res<Time>,
    mut commands: Commands,
    mut fruits: Query<(Entity, &Fruit, &mut FruitGrowthState)>,
    fruit_map: Res<FruitSpeciesMap>,
) {
    for (fruit_ent, fruit, mut growth) in fruits.iter_mut() {
        let fruit_type = fruit_map
            .species_vector
            .get(fruit.fruit_type)
            .expect(&format!("Unknown fruit type {}", fruit.fruit_type));
        match *growth {
            FruitGrowthState::Empty {
                ref mut seconds_of_growth,
            } => {
                *seconds_of_growth += time.delta_seconds();
                if *seconds_of_growth >= fruit_type.growth_time_secs {
                    commands
                        .entity(fruit_ent)
                        .insert((FruitGrowthState::Fruited, fruit_type.fruit_image.clone()));
                }
            }
            FruitGrowthState::Fruited => (),
        }
    }
}

pub fn obs_fruit_harvested(
    event: Trigger<HarvestFruitEvent>,
    mut score: ResMut<super::Score>,
    mut commands: Commands,
) {
    info!("Triggered fruit harvest");
    score.0 += 1;
    commands
        .entity(event.entity())
        .insert((FruitGrowthState::Empty {
            seconds_of_growth: 0.0,
        },))
        .remove::<Handle<Image>>();
}
