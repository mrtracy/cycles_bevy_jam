use std::any::TypeId;

use bevy::prelude::*;
use bevy_mod_picking::{events::Pointer, prelude::On, selection::Select, PickableBundle};

use crate::{fruit_type::FruitGenus, ui::CurrentIntention, units::DebugPlantType};

#[derive(Component)]
pub struct Tree {
    #[allow(dead_code)]
    pub genus: FruitGenus,
}

impl Tree {
    pub fn new_bundle(texture: Handle<Image>) -> impl Bundle {
        (
            Tree {
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
        )
    }
}
