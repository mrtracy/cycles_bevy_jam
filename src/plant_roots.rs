use bevy::prelude::*;
use bevy_mod_picking::{events::Pointer, prelude::On, selection::Select, PickableBundle};

use crate::{fruit_type::FruitGenus, ui::CurrentInspectedUnit, LevelBounds};

#[derive(Component)]
pub struct Plant {
    #[allow(dead_code)]
    pub genus: FruitGenus,
}

impl Plant {
    pub fn new_bundle(texture: Handle<Image>) -> impl Bundle {
        (
            Plant {
                genus: FruitGenus::Carrot,
            },
            texture,
            Sprite::default(),
            PickableBundle::default(),
            On::<Pointer<Select>>::commands_mut(|event, commands| {
                commands.insert_resource(CurrentInspectedUnit::Tree(event.target));
            }),
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
