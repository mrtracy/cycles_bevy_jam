use bevy::prelude::*;
use bevy_mod_picking::pointer::PointerId;

use crate::{buildings::SpriteData, ui::CurrentIntention, CameraPointerParam, LevelBounds};

#[derive(Component)]
pub struct BuildingPreview;

pub fn sys_hover_building_effect(
    mut commands: Commands,
    pointers: CameraPointerParam,
    current_inspector: Res<CurrentIntention>,
    bounds: Res<LevelBounds>,
    mut building_preview_query: Query<(Entity, &mut Transform), With<BuildingPreview>>,
    building_data: Query<&SpriteData>,
) {
    let CurrentIntention::Prospective(ent) = *current_inspector else {
        if let Ok((entity, _)) = building_preview_query.get_single() {
            commands.entity(entity).despawn();
        };
        return;
    };

    let Some(pos) = pointers.get_world_pointer_location(PointerId::Mouse) else {
        return;
    };
    if !bounds.in_bounds(pos) {
        return;
    }
    let Ok(sprite_data) = building_data.get(ent) else {
        warn!("Sprite data was not found for prospective entity type");
        return;
    };
    match building_preview_query.get_single_mut().ok() {
        Some((_, mut transform)) => {
            *transform = Transform::from_xyz(pos.x, pos.y, 0.);
        }
        None => {
            commands.spawn((
                BuildingPreview,
                SpriteBundle {
                    texture: sprite_data.primary_sprite.clone(),
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

pub fn building_preview_active(res: Res<CurrentIntention>) -> bool {
    matches!(*res, CurrentIntention::Prospective(..))
}

impl Plugin for BuildingPreviewPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            sys_hover_building_effect
                .run_if(building_preview_active.or_else(resource_changed::<CurrentIntention>)),
        );
    }
}
