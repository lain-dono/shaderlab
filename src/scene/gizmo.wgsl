struct View {
    view_proj: mat4x4<f32>,
    view: mat4x4<f32>,
    inverse_view: mat4x4<f32>,
    projection: mat4x4<f32>,
    world_position: vec3<f32>,

    width: f32,
    height: f32,
}

@group(0) @binding(0) var<uniform> view: View;

struct LineInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
}

struct LineOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn vs_main_line(in: LineInput) -> LineOutput {
    let position = view.view_proj * vec4<f32>(in.position, 1.0);
    return LineOutput(position, in.color);
}

@fragment
fn fs_main_line(in: LineOutput) -> @location(0) vec4<f32> {
    return in.color;
}


// https://asliceofrendering.com/scene%20helper/2020/01/05/InfiniteGrid/

fn inverse4x4(m: mat4x4<f32>) -> mat4x4<f32> {
    let a00 = m[0][0]; let a01 = m[0][1]; let a02 = m[0][2]; let a03 = m[0][3];
    let a10 = m[1][0]; let a11 = m[1][1]; let a12 = m[1][2]; let a13 = m[1][3];
    let a20 = m[2][0]; let a21 = m[2][1]; let a22 = m[2][2]; let a23 = m[2][3];
    let a30 = m[3][0]; let a31 = m[3][1]; let a32 = m[3][2]; let a33 = m[3][3];

    let b00 = a00 * a11 - a01 * a10;
    let b01 = a00 * a12 - a02 * a10;
    let b02 = a00 * a13 - a03 * a10;
    let b03 = a01 * a12 - a02 * a11;
    let b04 = a01 * a13 - a03 * a11;
    let b05 = a02 * a13 - a03 * a12;
    let b06 = a20 * a31 - a21 * a30;
    let b07 = a20 * a32 - a22 * a30;
    let b08 = a20 * a33 - a23 * a30;
    let b09 = a21 * a32 - a22 * a31;
    let b10 = a21 * a33 - a23 * a31;
    let b11 = a22 * a33 - a23 * a32;

    let det = b00 * b11 - b01 * b10 + b02 * b09 + b03 * b08 - b04 * b07 + b05 * b06;

    return mat4x4<f32>(
        (a11 * b11 - a12 * b10 + a13 * b09) / det,
        (a02 * b10 - a01 * b11 - a03 * b09) / det,
        (a31 * b05 - a32 * b04 + a33 * b03) / det,
        (a22 * b04 - a21 * b05 - a23 * b03) / det,
        (a12 * b08 - a10 * b11 - a13 * b07) / det,
        (a00 * b11 - a02 * b08 + a03 * b07) / det,
        (a32 * b02 - a30 * b05 - a33 * b01) / det,
        (a20 * b05 - a22 * b02 + a23 * b01) / det,
        (a10 * b10 - a11 * b08 + a13 * b06) / det,
        (a01 * b08 - a00 * b10 - a03 * b06) / det,
        (a30 * b04 - a31 * b02 + a33 * b00) / det,
        (a21 * b02 - a20 * b04 - a23 * b00) / det,
        (a11 * b07 - a10 * b09 - a12 * b06) / det,
        (a00 * b09 - a01 * b07 + a02 * b06) / det,
        (a31 * b01 - a30 * b03 - a32 * b00) / det,
        (a20 * b03 - a21 * b01 + a22 * b00) / det,
    );
}

struct GridOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) near: vec3<f32>,
    @location(1) far: vec3<f32>,
    @location(2) uv: vec2<f32>,
}

fn unproject_point(x: f32, y: f32, z: f32) -> vec3<f32> {
    let unprojected = inverse4x4(view.view_proj) * vec4<f32>(x, y, z, 1.0);
    return unprojected.xyz / unprojected.w;
}

fn resolution() -> vec2<f32> {
    return vec2<f32>(view.width, view.height);
}

@vertex
fn vs_main_grid(@builtin(vertex_index) in_vertex_index: u32) -> GridOutput {
    let u = f32((in_vertex_index << 1u) & 2u);
    let v = f32(in_vertex_index & 2u);
    let u = u - 1.0;
    let v = 1.0 - v;

    return GridOutput(
        vec4<f32>(u, v, 0.0, 1.0),
        unproject_point(u, v, 1.000),
        unproject_point(u, v, 0.0000001),
        vec2<f32>(u, v),
    );
}

fn grid(pos: vec3<f32>, scale: f32, axis: bool) -> vec4<f32> {
    let coord = pos.xz * scale; // use the scale variable to set the distance between the lines
    let derivative = fwidth(coord);
    let grid = abs(fract(coord - 0.5) - 0.5) / derivative;
    let grid_line = min(grid.x, grid.y);
    let minimumz = min(derivative.y, 1.0);
    let minimumx = min(derivative.x, 1.0);
    let alpha = 1.0 - min(grid_line, 1.0);
    var color = vec4<f32>(0.01) * alpha;
    if (axis) {
        let extra = 1.0 / scale;
        // z axis
        if (pos.x > -extra * minimumx && pos.x < extra * minimumx) {
            color.x = 0.0;
            color.y = 0.0;
            color.z = 0.1 * alpha;
            //color.w = 1.0;
        }
        // x axis
        if (pos.z > -extra * minimumz && pos.z < extra * minimumz) {
            color.x = 0.1 * alpha;
            color.y = 0.0;
            color.z = 0.0;
            //color.w = 1.0;
        }
    }

    return color;
}

struct FragOut {
    @builtin(frag_depth) depth: f32,
    @location(0) color: vec4<f32>,
}

@fragment
fn fs_main_grid(in: GridOutput) -> FragOut {
    let t = -in.near.y / (in.far.y - in.near.y);
    let pos = in.near + t * (in.far - in.near);

    let near = 0.0005;
    let clip = view.view_proj * vec4<f32>(pos.xyz, 1.0);
    let depth = clip.z / clip.w;
    let fading = 1.0 - near / depth;

    let color = (grid(pos, 1.0, false) + grid(pos, 0.1, true)) * f32(t > 0.0);
    return FragOut(depth, color * fading);
}