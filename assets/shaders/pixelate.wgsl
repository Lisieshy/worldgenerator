#import bevy_pbr::utils
// #import bevy_pbr::mesh_types
// #import bevy_pbr::mesh_view_bindings
#import bevy_sprite::mesh2d_view_bindings

@group(1) @binding(0)
var<uniform> pixel_size: vec2<f32>;

@group(1) @binding(1)
var texture: texture_2d<f32>;

@group(1) @binding(2)
var our_sampler: sampler;

@fragment
fn fragment(
    @builtin(position) position: vec4<f32>,
    #import bevy_sprite::mesh2d_vertex_output
) -> @location(0) vec4<f32> {
    let uv: vec2<f32> = coords_to_viewport_uv(position.xy, view.viewport);

    let pxsize: vec2<f32> = pixel_size / view.viewport.zw;
    let fixedUV: vec2<f32> = uv + pxsize / 2.0;
    let pxUV: vec2<f32> = floor(fixedUV / pxsize) * pxsize;
    let col: vec3<f32> = textureSample(texture, our_sampler, uv).rgb;
    let pxCol: vec3<f32> = textureSample(texture, our_sampler, pxUV).rgb;
    let col0: vec3<f32> = vec3<f32>(mix(col.rgb, pxCol.rgb, 1.0));
    return vec4<f32>(col0, 1.0);
}
