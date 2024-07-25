use std::any::TypeId;

use bevy::prelude::*;
use bevy_mod_picking::{events::Pointer, prelude::On, selection::Select, PickableBundle};

use crate::{
    fruit_type::FruitGenus,
    ui::CurrentIntention,
    units::{DebugPlantType, PathFollower},
};

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
                commands.insert_resource(CurrentIntention::Inspect(
                    TypeId::of::<DebugPlantType>(),
                    event.target,
                ));
            }),
            PathFollower {
                current_dist: 0.0,
                speed: 5.0,
            },
        )
    }
}
