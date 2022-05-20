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

[[group(0), binding(0)]] var<uniform> view: View;

struct LineInput {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] color: vec4<f32>;
};

struct LineOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] color: vec4<f32>;
};

[[stage(vertex)]]
fn vs_main_line(in: LineInput) -> LineOutput {
    let position = view.view_proj * vec4<f32>(in.position, 1.0);
    return LineOutput(position, in.color);
}

[[stage(fragment)]]
fn fs_main_line(in: LineOutput) -> [[location(0)]] vec4<f32> {
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
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] near: vec3<f32>;
    [[location(1)]] far: vec3<f32>;
    [[location(2)]] uv: vec2<f32>;
};

    fn unproject_point(x: f32, y: f32, z: f32, view_inv: mat4x4<f32>, proj_inv: mat4x4<f32>) -> vec3<f32> {
        //let unprojected = view_inv * proj_inv * vec4<f32>(x, y, z, 1.0);
        let unprojected = inverse4x4(view.view_proj) * vec4<f32>(x, y, z, 1.0);
        return unprojected.xyz / unprojected.w;
    }


    fn checkerboard(r: vec2<f32>, scale: f32) -> f32 {
        return f32((i32(floor(r.x / scale)) + i32(floor(r.y / scale))) % 2);
    }



    // https://www.shadertoy.com/view/XtBfzz

    // --- analytically box-filtered grid ---

fn resolution() -> vec2<f32> {
    return vec2<f32>(view.width, view.height);
}

    let N: f32 = 500.0; // grid ratio

    fn gridTextureGradBox(p: vec2<f32>, ddx: vec2<f32>, ddy: vec2<f32>) -> f32 {
        // filter kernel
        let w = max(abs(ddx), abs(ddy)) + 0.01;

        // analytic (box) filtering
        let a = p + 0.5*w;
        let b = p - 0.5*w;
        let i = (floor(a) + min(fract(a) * N, vec2<f32>(1.0)) - floor(b) - min(fract(b) * N, vec2<f32>(1.0))) / (N * w);

        // pattern
        return (1.0 - i.x) * (1.0 - i.y);
    }


    struct Intersect {
        pos: vec3<f32>;
        nor: vec3<f32>;
        matid: i32;
        tmin: f32;
    };

    struct Ray {
        ro: vec3<f32>;
        rd: vec3<f32>;
    };

    fn intersect(ray: Ray) -> Intersect {
        // raytrace
        var out: Intersect;

        out.tmin = 10000.0;
        out.nor = vec3<f32>(0.0);
        out.pos = vec3<f32>(0.0);
        out.matid = -1;

        // raytrace-plane
        let h = (0.01-ray.ro.y) / ray.rd.y;
        if (h > 0.0) {
            out.tmin = h;
            out.nor = vec3<f32>(0.0, 1.0, 0.0);
            out.pos = ray.ro + h * ray.rd;
            out.matid = 0;
        }

        return out;
    }

    fn texCoords(pos: vec3<f32>, mid: i32) -> vec2<f32> {
        return 10.0 * pos.xz;
    }

    struct Camera {
        ro: vec3<f32>;
        ta: vec3<f32>;
    };

    fn calcCamera() -> Camera {
        let an = 0.1 * sin(0.2);
        return Camera(
            vec3<f32>( 5.0*cos(an), 0.5, 5.0*sin(an) ),
            vec3<f32>( 0.0, 1.0, 0.0 ),
        );
    }

    //===============================================================================================
    //===============================================================================================
    // render
    //===============================================================================================
    //===============================================================================================


    fn calcRayForPixel(pix: vec2<f32>) -> Ray {
        let p = (2.0*pix-resolution().xy) / resolution().y;

        // camera movement
        let camera = calcCamera();

        // camera matrix
        let ww = normalize(camera.ta - camera.ro);
        let uu = normalize(cross(ww, vec3<f32>(0.0,1.0,0.0)));
        let vv = normalize(cross(uu, ww));

        // create view ray
        let rd = normalize(p.x * uu + p.y * vv + 2.0 * ww);

        return Ray(camera.ro, rd);
    }

    fn mainImage(fragCoord: vec2<f32>) -> vec4<f32> {
        let p = (-resolution().xy + 2.0 * fragCoord) / resolution().y;

        let ray = calcRayForPixel(fragCoord + vec2<f32>(0.0, 0.0));
        let ddx = calcRayForPixel(fragCoord + vec2<f32>(1.0, 0.0));
        let ddy = calcRayForPixel(fragCoord + vec2<f32>(0.0, 1.0));

        let trace = intersect(ray);

        var col = vec3<f32>(0.0, 0.0, 0.0);
        var alpha = 0.0;
        if (trace.matid != -1) {
            // -----------------------------------------------------------------------
            // compute ray differentials by intersecting the tangent plane to the surface.
            // -----------------------------------------------------------------------

            // computer ray differentials
            let ddx_pos = ddx.ro - ddx.rd * dot(ddx.ro - trace.pos, trace.nor) / dot(ddx.rd, trace.nor);
            let ddy_pos = ddy.ro - ddy.rd * dot(ddy.ro - trace.pos, trace.nor) / dot(ddy.rd, trace.nor);

            // calc texture sampling footprint
            let     uv = texCoords(trace.pos, trace.matid);
            let ddx_uv = texCoords(ddx_pos, trace.matid) - uv;
            let ddy_uv = texCoords(ddy_pos, trace.matid) - uv;

            // shading
            let grid = gridTextureGradBox(uv, ddx_uv, ddy_uv);
            let mate = vec3<f32>(1.0, 1.0, 1.0) * (1.0 - grid);
            col = mate;

            // fog
            //let t = trace.tmin;
            //col = mix( col, vec3<f32>(0.9), 1.0 - exp(-0.00001*t*t) );

            //alpha = (1.0 - grid);
            alpha = (1.0 - grid);
        }

        // gamma correction
        //col = pow(col, vec3<f32>(0.4545));

        return vec4<f32>(col, alpha);
    }

fn to_finite(
    proj: mat4x4<f32>,
    z_near: f32,
    z_far: f32,
) -> mat4x4<f32> {
    let nmf = z_near - z_far;

    return mat4x4<f32>(
        proj[0],
        proj[1],
        //proj[2],
        //proj[3],
        //vec4<f32>(0.0, 0.0, -z_far / nmf - 1.0, -1.0),
        //vec4<f32>(0.0, 0.0, -z_near * z_far / nmf, 0.0),

        vec4<f32>(0.0, 0.0, 0.0, -1.0),
        vec4<f32>(0.0, 0.0, z_near, 0.0),
    );
}

[[stage(vertex)]]
fn vs_main_grid([[builtin(vertex_index)]] in_vertex_index: u32) -> GridOutput {
    let u = f32((in_vertex_index << 1u) & 2u);
    let v = f32(in_vertex_index & 2u);
    let u = u - 1.0;
    let v = 1.0 - v;

    //let view_inv = inverse4x4(view.view);
    let view_inv = view.view;
    //let proj_inv = inverse4x4(to_finite(view.projection, view.near, view.far));
    let proj_inv = to_finite(view.projection, view.near, view.far);
    //let proj_inv = view.projection;

    return GridOutput(
        vec4<f32>(u, v, 0.0, 1.0),
        unproject_point(u, v, 1.000, view_inv, proj_inv),
        unproject_point(u, v, 0.0000001, view_inv, proj_inv),
        //unproject_point(u, v, 0.00000, view_inv, proj_inv),
        vec2<f32>(u, v),
    );
}

fn grid(pos: vec3<f32>, scale: f32, axis: bool) -> vec4<f32> {
    let coord = pos.xz * scale; // use the scale variable to set the distance between the lines
    let derivative = fwidth(coord);
    let grid = abs(fract(coord - 0.5) - 0.5) / derivative;
    let line = min(grid.x, grid.y);
    let minimumz = min(derivative.y, 1.0);
    let minimumx = min(derivative.x, 1.0);
    let alpha = 1.0 - min(line, 1.0);
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

fn computeDepth(pos: vec3<f32>) -> f32 {
    let clip_space_pos = view.view_proj * vec4<f32>(pos.xyz, 1.0);
    return (clip_space_pos.z / clip_space_pos.w);
}

fn computeLinearDepth(pos: vec3<f32>) -> f32 {
    let clip_space_pos = view.view_proj * vec4<f32>(pos.xyz, 1.0);
    let clip_space_depth = (clip_space_pos.z / clip_space_pos.w); // put back between -1 and 1
    return clip_space_depth; // normalize
}

struct FragOut {
    [[builtin(frag_depth)]] depth: f32;
    [[location(0)]] color: vec4<f32>;
};

fn remap(value: f32, in: vec2<f32>, out: vec2<f32>) -> f32 {
    return out.x + (value - in.x) * (out.y - out.x) / (in.y - in.x);
}


[[stage(fragment)]]
fn fs_main_grid(in: GridOutput) -> FragOut {
    let t = -in.near.y / (in.far.y - in.near.y);
    let pos = in.near + t * (in.far - in.near);

    let near = 0.0005;
    let clip = view.view_proj * vec4<f32>(pos.xyz, 1.0);
    let fading = 1.0 - near / (clip.z / clip.w);

    let color = (grid(pos, 1.0, false) + grid(pos, 0.1, true)) * f32(t > 0.0);
    return FragOut(computeDepth(pos), color * fading);
}