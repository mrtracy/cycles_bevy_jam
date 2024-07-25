use super::GameState;
use bevy::prelude::*;

// Level loading process:
//  - Load image asset
//  - Once loaded, convert to tilemap structure
//  - Fire loaded event.
//  - On loaded:
//    - Game Mode to Playing
//

#[derive(Component)]
pub struct CurrentLevel;

#[derive(Resource)]
pub struct LoadingLevel(pub Handle<Image>);

pub(crate) fn kickoff_load(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(LoadingLevel(asset_server.load("levels/level1.png")));
}

pub(crate) fn sys_wait_for_loading_level(
    mut commands: Commands,
    loading_level: Res<LoadingLevel>,
    images: Res<Assets<Image>>,
    asset_server: Res<AssetServer>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    use bevy_ecs_tilemap::prelude::*;

    let level_data_handle = &loading_level.0;

    let Some(image_data) = images.get(level_data_handle) else {
        return;
    };

    let texture_handle: Handle<Image> = asset_server.load("tiles.png");
    let map_size = TilemapSize {
        x: image_data.width(),
        y: image_data.height(),
    };

    let tilemap_entity = commands.spawn_empty().id();

    let mut tile_storage = TileStorage::empty(map_size);

    let grayscale = image_data.clone().try_into_dynamic().unwrap().to_luma8();

    for x in 0..map_size.x {
        for y in 0..map_size.y {
            let tile_pos = TilePos { x, y };
            let tile_entity = commands
                .spawn(TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    texture_index: match grayscale.get_pixel(x, y).0[0] {
                        x if x < 50 => TileTextureIndex(0),
                        _ => TileTextureIndex(1),
                    },
                    ..Default::default()
                })
                .id();
            tile_storage.set(&tile_pos, tile_entity);
        }
    }

    let tile_size = TilemapTileSize { x: 16.0, y: 16.0 };
    let grid_size = tile_size.into();
    let map_type = TilemapType::default();

    commands.entity(tilemap_entity).insert((
        TilemapBundle {
            grid_size,
            map_type,
            size: map_size,
            storage: tile_storage,
            texture: TilemapTexture::Single(texture_handle),
            tile_size,
            transform: get_tilemap_center_transform(&map_size, &grid_size, &map_type, -1.0),
            ..Default::default()
        },
        CurrentLevel,
    ));

    commands.remove_resource::<LoadingLevel>();
    next_game_state.set(GameState::Playing);
}