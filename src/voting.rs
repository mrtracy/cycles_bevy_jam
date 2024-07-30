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
    input::{keyboard::KeyCode, ButtonInput},
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
use bevy_rapier2d::{
    dynamics::{Damping, GravityScale, RigidBody},
    geometry::Collider,
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
            (sys_draw_guards, sys_move_draw_player).run_if(
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
            RigidBody::Dynamic,
            Collider::ball(10.),
            Damping {
                linear_damping: 2.0,
                angular_damping: 2.0,
            },
            GravityScale(0.),
            Guard::default(),
        )
    }
}

#[derive(Component, Default)]
struct Player {}

impl Player {
    fn instantiate(position: Vec2) -> impl Bundle {
        (
            SpatialBundle {
                transform: Transform {
                    translation: (position, 0.).into(),
                    ..Default::default()
                },
                ..Default::default()
            },
            RigidBody::KinematicPositionBased,
            Collider::ball(10.),
            GravityScale(0.),
            Player::default(),
        )
    }
}

fn sys_init_level(mut next_play_state: ResMut<NextState<GameState>>, mut commands: Commands) {
    commands.spawn(Guard::instantiate(Vec2 { x: 20., y: 20. }));
    commands.spawn(Player::instantiate(Vec2 { x: 0., y: 0. }));
    next_play_state.set(GameState::Playing);
}

fn sys_move_draw_player(
    mut player: Query<(&mut Transform, &InheritedVisibility), With<Player>>,
    mut gizmos: gizmos::Gizmos,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let (mut transform, visibility) = player.single_mut();
    if keys.pressed(KeyCode::KeyW) {
        transform.translation.y += 1.;
    }
    if keys.pressed(KeyCode::KeyS) {
        transform.translation.y -= 1.;
    }
    if keys.pressed(KeyCode::KeyD) {
        transform.translation.x += 1.;
    }
    if keys.pressed(KeyCode::KeyA) {
        transform.translation.x -= 1.;
    }

    if *visibility == InheritedVisibility::VISIBLE {
        gizmos.circle_2d(
            transform.translation.xy(),
            10.,
            Color::linear_rgb(0., 1., 0.),
        );
    }
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
                Color::linear_rgb(0., 0., 1.0),
            );
        }
    }
}
