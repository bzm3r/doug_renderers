// courtesy of robswain: https://github.com/superdump/bevy-vertex-pulling
struct View {
    view_proj: mat4x4<f32>;
    view: mat4x4<f32>;
    inverse_view: mat4x4<f32>;
    projection: mat4x4<f32>;
    world_pos: vec3<f32>;
    near: f32;
    far: f32;
    width: f32;
    height: f32;
};

struct Quad {
    p0: vec2<f32>;
    p1: vec2<f32>;
    layer: f32;
    color: u32;
};

struct Quads {
    data: array<Quad>;
};

struct Palette {
    colors: array<vec4<f32>>;
};

[[group(0), binding(0)]]
var<uniform> view: View;

[[group(1), binding(0)]]
var<storage> quads: Quads;

[[group(1), binding(1)]]
var<storage> palette: Palette;

struct VertexOutput {
    [[builtin(position)]] screen_pos: vec4<f32>;
    [[location(0)]] d_bot_left: vec2<f32>;
    [[location(1)]] d_top_right: vec2<f32>;
    [[location(2)]] color: vec4<f32>;
};

[[stage(vertex)]]
fn vertex([[builtin(vertex_index)]] vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;

    // x >> 2, divides x by 2
    let instance_index = vertex_index >> 2u;
    let quad = quads.data[instance_index];

    let xyz = vec3<i32>(i32(vertex_index & 0x1u), i32((vertex_index & 0x2u) >> 1u), 0);
    let uv = vec2<f32>(xyz.xy);
    let wh = quad.p1.xy - quad.p0.xy;
    let relative_pos = vec2<f32>(uv * wh);

    let world_pos = vec4<f32>(quad.p0.xy + relative_pos, quad.layer, 1.0);
    //out.world_pos = vec4<f32>(quad.p0.xy + relative_pos, quad.layer, 1.0);
    //out.world_normal = vec3<f32>(0.0, 0.0, 1.0);

    out.d_bot_left = vec2<f32>(world_pos.xy - quad.p0);
    out.d_top_right = vec2<f32>(quad.p1 - world_pos.xy);
    out.screen_pos = view.view_proj * world_pos;
    out.color = palette.colors[0];
    //vec4<f32>(1.0, 1.0, 1.0, 1.0);
    // palette[quad.color];
    return out;
}

struct FragmentInput {
    [[builtin(front_facing)]] is_front: bool;
    [[builtin(position)]] screen_pos: vec4<f32>;
    [[location(0)]] d_bot_left: vec2<f32>;
    [[location(1)]] d_top_right: vec2<f32>;
    [[location(2)]] color: vec4<f32>;
};

[[stage(fragment)]]
fn fragment(in: FragmentInput) -> [[location(0)]] vec4<f32> {
    var local_color = in.color;
    let t = 1.0;
    let blur = t;
    let abs_zoom = abs(in.screen_pos.z);
    let zoomed_blur = smoothStep(0.2, 0.6, abs_zoom) * blur;
    let min_d = min(min(in.d_bot_left.x, in.d_bot_left.y), min(in.d_top_right.x, in.d_top_right.y));
    let alpha = max(1.0 - smoothStep(t - zoomed_blur, t + zoomed_blur, min_d), 0.2);
    let c = vec4<f32>(local_color.xyz, alpha);
    return c;
}

