struct VertexOutput {
    [[location(0)]] uv: vec2<f32>;
    [[builtin(position)]] pos: vec4<f32>;
};

[[block]]
struct Globals {
    camera_view: mat4x4<f32>;
    camera_proj: mat4x4<f32>;
};
[[group(0), binding(1)]]
var globals: Globals;

[[block]]
struct Object {
    transform: mat4x4<f32>;
    rgb: vec3<f32>;
};
[[group(1), binding(2)]]
var object: Object;

[[stage(vertex)]]
fn vs(
    [[location(0)]] pos: vec4<f32>,
    [[location(1)]] uv: vec2<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.uv = uv;
    var world_pos: vec4<f32> = object.transform * pos;
    out.pos = globals.camera_proj * globals.camera_view * world_pos;
    return out;
}

[[group(1), binding(0)]]
var albedo: texture_2d<f32>;

[[group(0), binding(1)]]
var s: sampler;

[[stage(fragment)]]
fn fs_fill(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return textureSample(albedo, s, in.uv);
}

[[stage(fragment)]]
fn fs_wire() -> [[location(0)]] vec4<f32> {
    return vec4<f32>(0.0, 0.5, 0.0, 0.5);
}
