use bevy::prelude::*;

use crate::{Score, SpatialTracked};

#[derive(Event)]
pub struct HarvestFruitEvent {
    #[allow(dead_code)]
    pub harvester_ent: Entity,
}

#[derive(Component)]
pub struct Fruit;

#[derive(Component)]
pub enum FruitGrowthState {
    Empty { seconds_remaining: f32 },
    Fruited,
}

impl Fruit {
    pub fn new_bundle(texture: Handle<Image>, loc: Vec2) -> impl Bundle {
        (
            SpatialTracked,
            SpriteBundle {
                texture,
                transform: Transform::from_xyz(loc.x, loc.y, 0.0),
                visibility: Visibility::Hidden,
                ..Default::default()
            },
            FruitGrowthState::Empty {
                seconds_remaining: 6.0,
            },
        )
    }
}

pub fn sys_fruit_grow(
    time: Res<Time>,
    mut commands: Commands,
    mut fruits: Query<(Entity, &mut FruitGrowthState)>,
) {
    for (fruit_ent, mut growth) in fruits.iter_mut() {
        match *growth {
            FruitGrowthState::Empty {
                seconds_remaining: ref mut ticks_remaining,
            } => {
                *ticks_remaining -= time.delta_seconds();
                if *ticks_remaining <= 0.0 {
                    commands
                        .entity(fruit_ent)
                        .insert((FruitGrowthState::Fruited, Visibility::Inherited));
                }
            }
            FruitGrowthState::Fruited => (),
        }
    }
}

pub fn obs_fruit_harvested(
    event: Trigger<HarvestFruitEvent>,
    mut score: ResMut<Score>,
    mut commands: Commands,
) {
    info!("Triggered fruit harvest");

    **score += 1;
    commands.entity(event.entity()).insert((
        FruitGrowthState::Empty {
            seconds_remaining: 6.0,
        },
        Visibility::Hidden,
    ));
}
