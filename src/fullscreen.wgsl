struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(1) uv: vec2<f32>,
};


@vertex
fn vs(@builtin(vertex_index) index: u32) -> VertexOutput {
    var out: VertexOutput;

    if (index == 0u) {
        out.clip_position = vec4<f32>(1.0, -1.0, 0.0, 1.0);
        out.uv = vec2<f32>(1.0, 1.0);
    } else if (index == 1u) {
        out.clip_position = vec4<f32>(1.0, 1.0, 0.0, 1.0);
        out.uv = vec2<f32>(1.0, 0.0);
    } else if (index == 2u) {
        out.clip_position = vec4<f32>(-1.0, 1.0, 0.0, 1.0);
        out.uv = vec2<f32>(0.0, 0.0);
    } else if (index == 3u) {
        out.clip_position = vec4<f32>(-1.0, 1.0, 0.0, 1.0);
        out.uv = vec2<f32>(0.0, 0.0);
    } else if (index == 4u) {
        out.clip_position = vec4<f32>(-1.0, -1.0, 0.0, 1.0);
        out.uv = vec2<f32>(0.0, 1.0);
    } else if (index == 5u) {
        out.clip_position = vec4<f32>(1.0, -1.0, 0.0, 1.0);
        out.uv = vec2<f32>(1.0, 1.0);
    }

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
fn fs(in: VertexOutput) -> FragmentOutput {
    var out: FragmentOutput;

    out.color = textureSampleLevel(fullscreen_tex, fullscreen_tex_sampler, in.uv, 0.0);

    return out;
}

