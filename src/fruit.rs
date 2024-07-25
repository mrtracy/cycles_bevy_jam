use bevy::{math::vec2, prelude::*};

use crate::{fruit_type::FruitSpeciesMap, SpatialTracked};

#[derive(Component)]
pub struct FruitBranch {
    pub species: usize,
}

#[derive(Component)]
pub enum FruitBranchAttachment {
    #[allow(dead_code)]
    Fruit(Entity),
}

#[derive(Bundle)]
pub struct FruitBranchBundle {
    pub branch: FruitBranch,
    pub sprite: SpriteBundle,
}

pub fn sys_fruit_branch_spawn_fruit(
    mut commands: Commands,
    plants: Query<(Entity, &FruitBranch), Without<FruitBranchAttachment>>,
) {
    for (branch_ent, branch) in plants.iter() {
        let fruit_id = commands
            .spawn(Fruit::new_bundle(branch.species, vec2(1.0, 1.0)))
            .set_parent(branch_ent)
            .id();
        commands
            .entity(branch_ent)
            .insert(FruitBranchAttachment::Fruit(fruit_id));
    }
}

#[derive(Component)]
pub struct Fruit {
    fruit_type: usize,
}

#[derive(Component)]
pub enum FruitGrowthState {
    Bud { seconds_of_growth: f32 },
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
            FruitGrowthState::Bud {
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
            .unwrap_or_else(|| panic!("Unknown fruit type {}", fruit.fruit_type));
        match *growth {
            FruitGrowthState::Bud {
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

#[derive(Event)]
pub struct HarvestFruitEvent {
    #[allow(dead_code)]
    pub harvester_ent: Entity,
}

pub fn obs_fruit_harvested(
    event: Trigger<HarvestFruitEvent>,
    mut score: ResMut<super::Score>,
    mut commands: Commands,
    fruit_query: Query<&Parent, With<Fruit>>,
) {
    let target_fruit = event.entity();
    let Ok(parent_branch_ent) = fruit_query.get(event.entity()) else {
        return;
    };

    score.0 += 1;
    commands.entity(target_fruit).despawn();
    commands
        .entity(parent_branch_ent.get())
        .remove::<FruitBranchAttachment>();
}
