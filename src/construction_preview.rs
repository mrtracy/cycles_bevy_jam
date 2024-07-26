use bevy::prelude::*;
use bevy_mod_picking::pointer::PointerId;

use crate::{
    ui::CurrentIntention, units::BuildingTypeMap, CameraPointerParam, MapQuery, MapQueryHelpers,
};

#[derive(Component)]
pub struct BuildingPreview;

pub fn sys_hover_building_effect(
    mut commands: Commands,
    pointers: CameraPointerParam,
    current_inspector: Res<CurrentIntention>,
    map_query: MapQuery,
    mut building_preview_query: Query<(Entity, &mut Transform), With<BuildingPreview>>,
    building_types: Res<BuildingTypeMap>,
) {
    let CurrentIntention::Prospective(typ) = *current_inspector else {
        if let Ok((entity, _)) = building_preview_query.get_single() {
            commands.entity(entity).despawn();
        };
        return;
    };

    let Some(pos) = pointers.get_world_pointer_location(PointerId::Mouse) else {
        return;
    };
    let Some(snapped_pos) = map_query.snap_to_tile_center(&pos) else {
        return;
    };

    let Some(building_type) = building_types.type_map.get(&typ) else {
        warn!("Sprite data was not found for prospective entity type");
        return;
    };
    match building_preview_query.get_single_mut().ok() {
        Some((_, mut transform)) => {
            *transform = Transform::from_translation(snapped_pos);
        }
        None => {
            commands.spawn((
                BuildingPreview,
                SpriteBundle {
                    texture: building_type.sprite_image().clone(),
                    transform: Transform::from_translation(snapped_pos),
                    sprite: Sprite {
                        color: Color::linear_rgba(0.2, 0.3, 1.0, 0.4),
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
