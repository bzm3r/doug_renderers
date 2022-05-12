// courtesy of robswain: https://github.com/superdump/bevy-vertex-pulling
struct View {
    view_proj: mat4x4<f32>;
    view: mat4x4<f32>;
    inverse_view: mat4x4<f32>;
    projection: mat4x4<f32>;
    world_position: vec3<f32>;
    near: f32;
    far: f32;
    width: f32;
    height: f32;
};

struct Quad {
    p0: vec4<f32>;
    p1: vec4<f32>;
};

struct Quads {
    data: array<Quad>;
};

[[group(0), binding(0)]]
var<uniform> view: View;

[[group(1), binding(0)]]
var<storage> quads: Quads;

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] world_position: vec4<f32>;
    [[location(1)]] world_normal: vec3<f32>;
    [[location(2)]] uv: vec2<f32>;
    [[location(3)]] color: vec4<f32>;
};

[[stage(vertex)]]
fn vertex([[builtin(vertex_index)]] vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;

    // x >> 2, divides x by 2
    let instance_index = vertex_index >> 2u;
    let quad = quads.data[instance_index];

    let xyz = vec3<i32>(i32(vertex_index & 0x1u), i32((vertex_index & 0x2u) >> 1u), 0);
    out.uv = vec2<f32>(xyz.xy);
    let wh = quad.p1.xy - quad.p0.xy;
    let relative_pos = vec2<f32>(out.uv * wh);

    out.world_position = vec4<f32>(quad.p0.xy + relative_pos, quad.p1.z, 1.0);
    out.world_normal = vec3<f32>(0.0, 0.0, 1.0);

    out.clip_position = view.view_proj * out.world_position;
    out.color = vec4<f32>(1.0, 1.0, 1.0, 1.0);
    return out;
}

struct FragmentInput {
    [[builtin(front_facing)]] is_front: bool;
    [[location(0)]] world_position: vec4<f32>;
    [[location(1)]] world_normal: vec3<f32>;
    [[location(2)]] uv: vec2<f32>;
    [[location(3)]] color: vec4<f32>;
};

[[stage(fragment)]]
fn fragment(in: FragmentInput) -> [[location(0)]] vec4<f32> {
    if (in.uv.x > 0.9 || in.uv.y > 0.9 || in.uv.x < 0.1 || in.uv.y < 0.1) {
        return vec4<f32>(1.0, 1.0, 1.0, 1.0);
    } else {
        return vec4<f32>(1.0, 0.0, 0.0, 0.1);
    }
}

