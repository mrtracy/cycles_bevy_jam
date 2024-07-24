use bevy::{prelude::*, window::PrimaryWindow};

use crate::{get_world_click_pos, ui::CurrentInspectedUnit, LevelBounds};

#[derive(Component)]
pub struct BuildingPreview;

pub fn sys_hover_building_effect(
    mut commands: Commands,
    current_inspector: Res<CurrentInspectedUnit>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
    bounds: Res<LevelBounds>,
    asset_server: Res<AssetServer>,
    mut building_preview_query: Query<(Entity, &mut Transform), With<BuildingPreview>>,
) {
    let CurrentInspectedUnit::Prospective(..) = *current_inspector else {
        if let Ok((entity, _)) = building_preview_query.get_single() {
            commands.entity(entity).despawn();
        };
        return;
    };

    let Some(pos) = get_world_click_pos(&q_windows, &camera) else {
        return;
    };
    if !bounds.in_bounds(pos) {
        return;
    }
    match building_preview_query.get_single_mut().ok() {
        Some((_, mut transform)) => {
            *transform = Transform::from_xyz(pos.x, pos.y, 0.);
        }
        None => {
            commands.spawn((
                BuildingPreview,
                SpriteBundle {
                    texture: asset_server.load("harvester_test.png"),
                    transform: Transform::from_xyz(pos.x, pos.y, 0.),
                    sprite: Sprite {
                        color: Color::linear_rgba(0.2, 0.3, 1.0, 0.4).into(),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            ));
        }
    }
}

pub struct BuildingPreviewPlugin;

pub fn building_preview_active(res: Res<CurrentInspectedUnit>) -> bool {
    matches!(*res, CurrentInspectedUnit::Prospective(..))
}

impl Plugin for BuildingPreviewPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            sys_hover_building_effect
                .run_if(building_preview_active.or_else(resource_changed::<CurrentInspectedUnit>)),
        );
    }
}