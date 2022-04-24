struct VertexInput {
    [[location(0)]] position: vec2<f32>;
    [[location(1)]] texcoord: vec2<f32>;
    [[location(2)]] color: vec4<f32>;
};

struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] color: vec4<f32>;
    [[location(1)]] texcoord: vec2<f32>;
};

struct Viewport {
    size: vec4<f32>;
};

[[group(0), binding(0)]] var<uniform> viewport: Viewport;
[[group(1), binding(0)]] var image_color: texture_2d<f32>;
[[group(1), binding(1)]] var image_sampler: sampler;

fn linear_from_srgb(srgb: vec3<f32>) -> vec3<f32> {
    let cutoff = srgb < vec3<f32>(0.04045);
    let lower = srgb / 12.92;
    let higher = pow((srgb + 0.055) / 1.055, vec3<f32>(2.4));
    return select(higher, lower, cutoff);
}

fn srgb_from_linear(linear: vec3<f32>) -> vec3<f32> {
    let cutoff = linear < vec3<f32>(0.0031308);
    let lower = linear * 12.92;
    let higher = pow(linear, vec3<f32>(1.0 / 2.4)) * 1.055 - 0.055;
    return select(higher, lower, cutoff);
}

[[stage(vertex)]]
fn vs_main_srgb(in: VertexInput) -> VertexOutput {
    let x = 2.0 * in.position.x * viewport.size.z - 1.0;
    let y = 1.0 - 2.0 * in.position.y * viewport.size.w;
    let color = vec4<f32>(linear_from_srgb(in.color.rgb), in.color.a);
    return VertexOutput(vec4<f32>(x, y, 0.0, 1.0), color, in.texcoord);
}

[[stage(vertex)]]
fn vs_main_linear(in: VertexInput) -> VertexOutput {
    let x = 2.0 * in.position.x * viewport.size.z - 1.0;
    let y = 1.0 - 2.0 * in.position.y * viewport.size.w;
    return VertexOutput(vec4<f32>(x, y, 0.0, 1.0), in.color, in.texcoord);
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return in.color * textureSample(image_color, image_sampler, in.texcoord);
}