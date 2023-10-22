struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(1) uv: vec2<f32>,
};

const POS = array<vec4<f32>, 6>(
    vec4<f32>(1.0, -1.0, 0.0, 1.0),
    vec4<f32>(1.0, 1.0, 0.0, 1.0),
    vec4<f32>(-1.0, 1.0, 0.0, 1.0),
    vec4<f32>(-1.0, 1.0, 0.0, 1.0),
    vec4<f32>(-1.0, -1.0, 0.0, 1.0),
    vec4<f32>(1.0, -1.0, 0.0, 1.0)
);

const UVS = array<vec2<f32>, 6>(
    vec2<f32>(1.0, 0.0),
    vec2<f32>(1.0, 1.0),
    vec2<f32>(0.0, 1.0),
    vec2<f32>(0.0, 1.0),
    vec2<f32>(0.0, 0.0),
    vec2<f32>(1.0, 0.0)
);

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> VertexOutput {
    var out: VertexOutput;

    out.clip_position = POS[index];
    out.uv = UVS[index];

    return out;
}

struct FragmentOutput {
    @location(0) color: vec4<f32>,
};

@group(0)
@binding(0)
var fullscreen_tex_sampler: sampler;
@group(0)
@binding(1)
var fullscreen_tex: texture_2d<f32>;

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    var out: FragmentOutput;

    out.color = textureSampleLevel(fullscreen_tex, fullscreen_tex_sampler, in.uv, 0.0);

    return out;
}

