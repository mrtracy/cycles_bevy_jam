#import bevy_ecs_tilemap::common::{
    process_fragment, tilemap_data, sprite_texture, sprite_sampler
};
#import bevy_ecs_tilemap::vertex_output::MeshVertexOutput
#import bevy_sprite::mesh2d_view_bindings::globals

struct MyMaterial {
    apply_color: u32,
};

@group(3) @binding(0)
var<uniform> material: MyMaterial;

fn hsv2rgb(c: vec3<f32>) -> vec3<f32>
{
    let K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    let p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, vec3(0.0), vec3(1.0)), c.y);
}

@fragment
fn fragment(in: MeshVertexOutput) -> @location(0) vec4<f32> {
    if material.apply_color == 0 {
        return process_fragment(in);
    }

    // Copied from Process fragment, we are going to use TileColor differently (additive instead of multiplicative).
    let half_texture_pixel_size_u = 0.5 / tilemap_data.texture_size.x;
    let half_texture_pixel_size_v = 0.5 / tilemap_data.texture_size.y;
    let half_tile_pixel_size_u = 0.5 / tilemap_data.tile_size.x;
    let half_tile_pixel_size_v = 0.5 / tilemap_data.tile_size.y;

    // Offset the UV 1/2 pixel from the sides of the tile, so that the sampler doesn't bleed onto
    // adjacent tiles at the edges.
    var uv_offset: vec2<f32> = vec2<f32>(0.0, 0.0);
    if (in.uv.z < half_tile_pixel_size_u) {
        uv_offset.x = half_texture_pixel_size_u;
    } else if (in.uv.z > (1.0 - half_tile_pixel_size_u)) {
        uv_offset.x = - half_texture_pixel_size_u;
    }
    if (in.uv.w < half_tile_pixel_size_v) {
        uv_offset.y = half_texture_pixel_size_v;
    } else if (in.uv.w > (1.0 - half_tile_pixel_size_v)) {
        uv_offset.y = - half_texture_pixel_size_v;
    }

    let color = textureSample(sprite_texture, sprite_sampler, in.uv.xy + uv_offset, in.tile_id);

    // luminance conversion
    let luminance = vec3<f32>(0.2126*color.r + 0.7152*color.g + 0.0722*color.b) / 2.0;
   
    return vec4<f32>(luminance, 1.0) + in.color;
}