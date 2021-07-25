[[stage(vertex)]]
fn vs_main([[builtin(vertex_index)]] vertex_index: u32) -> [[builtin(position)]] vec4<f32> {
    let uv = vec2<f32>(f32(i32((vertex_index << 1u) & 2u)), f32(i32(vertex_index & 2u)));
    return vec4<f32>(uv * vec2<f32>(2.0, -2.0) + vec2<f32>(-1.0, 1.0), 0.0, 1.0);
}

[[stage(fragment)]]
fn fs_main() -> [[location(0)]] vec4<f32> {
    return vec4<f32>(0.02, 0.02, 0.02, 1.0);
}