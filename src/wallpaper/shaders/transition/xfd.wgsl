struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) texture_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) texture_coords: vec2<f32>,
}

@vertex
fn vs_main(
    model: VertexInput
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(model.position, 1.0);
    out.texture_coords = model.texture_coords;
    return out;
}

@group(0) @binding(0)
var t_diffuse_old: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse_old: sampler;
@group(0) @binding(2)
var t_diffuse_new: texture_2d<f32>;
@group(0) @binding(3)
var s_diffuse_new: sampler;

@group(1) @binding(0)
var<uniform> progress: f32;

@fragment
fn fs_main(
    in: VertexOutput
) -> @location(0) vec4<f32> {
    let old_color = textureSample(t_diffuse_old, s_diffuse_old, in.texture_coords);
    let new_color = textureSample(t_diffuse_new, s_diffuse_new, in.texture_coords);
    return mix(old_color, new_color, progress);
}
