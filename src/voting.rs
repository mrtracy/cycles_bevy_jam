use bevy::{
    app::{Plugin, Update},
    color::Color,
    ecs::{
        bundle::Bundle,
        component::Component,
        query::With,
        schedule::{common_conditions::resource_equals, Condition, IntoSystemConfigs},
        system::{Commands, Query, Res, ResMut},
    },
    gizmos::gizmos,
    math::{Quat, Vec2, Vec3Swizzles},
    prelude::{default, SpatialBundle},
    render::view::{InheritedVisibility, Visibility},
    state::{
        app::AppExtStates,
        condition::in_state,
        state::{NextState, States},
    },
    transform::components::Transform,
};

use crate::GameState;

pub struct VotingPlugin;

impl Plugin for VotingPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(
            Update,
            (sys_init_level).run_if(
                in_state(super::GameState::Loading)
                    .and_then(move |res: Res<super::Level>| (*res).level == 1),
            ),
        )
        .add_systems(
            Update,
            (sys_draw_guards).run_if(
                in_state(super::GameState::Playing)
                    .and_then(move |res: Res<super::Level>| (*res).level == 1),
            ),
        );
    }
}

#[derive(Component, Default)]
struct Guard {}

impl Guard {
    fn instantiate(position: Vec2) -> impl Bundle {
        (
            SpatialBundle {
                transform: Transform {
                    translation: (position, 0.).into(),
                    ..Default::default()
                },
                ..Default::default()
            },
            Guard::default(),
        )
    }
}

fn sys_init_level(mut next_play_state: ResMut<NextState<GameState>>, mut commands: Commands) {
    commands.spawn(Guard::instantiate(Vec2 { x: 4., y: 3. }));
    next_play_state.set(GameState::Playing);
}

fn sys_draw_guards(
    guards: Query<(&Transform, &InheritedVisibility), With<Guard>>,
    mut gizmos: gizmos::Gizmos,
) {
    for (transform, visibility) in guards.iter() {
        if *visibility == InheritedVisibility::VISIBLE {
            gizmos.circle_2d(
                transform.translation.xy(),
                10.,
                Color::linear_rgb(0.2, 0.1, 0.7),
            );
        }
    }
}
