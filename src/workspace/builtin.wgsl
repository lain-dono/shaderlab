fn builtin_blackbody(temperature: f32) -> vec3<f32> {
    var color = vec3<f32>(
        56100000.0 * pow(temperature, -1.5) + 148.0,
        100.04 * log(temperature) - 623.6,
        194.18 * log(temperature) - 1448.6,
    );

    if (temperature > 6500.0) {
        color.y = 35200000.0 * pow(temperature, -1.5) + 184.0;
    }

    color = clamp(color, vec3<f32>(0.0, 0.0, 0.0), vec3<f32>(255.0, 255.0, 255.0)) / vec3<f32>(255.0, 255.0, 255.0);

    if (temperature < 1000.0) {
        color = color * temperature / 1000.0;
    }

    return color;
}

fn builtin_inverse_lerp_1(a: f32, b: f32, t: f32) -> f32 {
    return (t - a)/(b - a);
}
fn builtin_inverse_lerp_2(a: vec2<f32>, b: vec2<f32>, t: vec2<f32>) -> vec2<f32> {
    return (t - a)/(b - a);
}
fn builtin_inverse_lerp_3(a: vec3<f32>, b: vec3<f32>, t: vec3<f32>) -> vec3<f32> {
    return (t - a)/(b - a);
}
fn builtin_inverse_lerp_4(a: vec4<f32>, b: vec4<f32>, t: vec4<f32>) -> vec4<f32> {
    return (t - a)/(b - a);
}

fn builtin_lerp_1(a: f32, b: f32, t: f32) -> f32 {
    return a + (b - a) * t;
}
fn builtin_lerp_2(a: vec2<f32>, b: vec2<f32>, t: vec2<f32>) -> vec2<f32> {
    return a + (b - a) * t;
}
fn builtin_lerp_3(a: vec3<f32>, b: vec3<f32>, t: vec3<f32>) -> vec3<f32> {
    return a + (b - a) * t;
}
fn builtin_lerp_4(a: vec4<f32>, b: vec4<f32>, t: vec4<f32>) -> vec4<f32> {
    return a + (b - a) * t;
}

fn builtin_random_range(seed: vec2<f32>, min: f32, max: f32) -> f32 {
    let rand = fract(sin(dot(seed, vec2<f32>(12.9898, 78.233))) * 43758.5453);
    return mix(min, max, rand);
}


fn builtin_remap_1(input: f32, in_range: vec2<f32>, out_range: vec2<f32>) -> f32 {
    return out_range.x + (input - in_range.x) * (out_range.y - out_range.x) / (in_range.y - in_range.x);
}
fn builtin_remap_2(input: vec2<f32>, in_range: vec2<f32>, out_range: vec2<f32>) -> vec2<f32> {
    return out_range.x + (input - in_range.x) * (out_range.y - out_range.x) / (in_range.y - in_range.x);
}
fn builtin_remap_3(input: vec3<f32>, in_range: vec2<f32>, out_range: vec2<f32>) -> vec3<f32> {
    return out_range.x + (input - in_range.x) * (out_range.y - out_range.x) / (in_range.y - in_range.x);
}
fn builtin_remap_4(input: vec4<f32>, in_range: vec2<f32>, out_range: vec2<f32>) -> vec4<f32> {
    return out_range.x + (input - in_range.x) * (out_range.y - out_range.x) / (in_range.y - in_range.x);
}

fn _gradient_noise_dir(p: vec2<f32>) -> vec2<f32> {
    let p = p % vec2<f32>(289.0);
    let x = (34.0 * p.x + 1.0) * p.x % 289.0 + p.y;
    let x = (34.0 * x + 1.0) * x % 289.0;
    let x = fract(x / 41.0) * 2.0 - 1.0;
    return normalize(vec2<f32>(x - floor(x + 0.5), abs(x) - 0.5));
}

fn _gradient_noise(p: vec2<f32>) -> f32 {
    let ip = floor(p);
    let fp = fract(p);
    let d00 = dot(_gradient_noise_dir(ip), fp);
    let d01 = dot(_gradient_noise_dir(ip + vec2<f32>(0.0, 1.0)), fp - vec2<f32>(0.0, 1.0));
    let d10 = dot(_gradient_noise_dir(ip + vec2<f32>(1.0, 0.0)), fp - vec2<f32>(1.0, 0.0));
    let d11 = dot(_gradient_noise_dir(ip + vec2<f32>(1.0, 1.0)), fp - vec2<f32>(1.0, 1.0));
    let fp = fp * fp * fp * (fp * (fp * 6.0 - 15.0) + 10.0);
    return mix(mix(d00, d01, fp.y), mix(d10, d11, fp.y), fp.x);
}

fn builtin_gradient_noise(uv: vec2<f32>, scale: f32) -> f32 {
    return _gradient_noise(uv * scale) + 0.5;
}

fn _random_01(uv: vec2<f32>) -> f32 {
    return fract(sin(dot(uv, vec2<f32>(12.9898, 78.233)))*43758.5453);
}

fn _value_noise(uv: vec2<f32>) -> f32 {
    let i = floor(uv);
    let f = fract(uv);
    let f = f * f * (3.0 - 2.0 * f);

    //let uv = abs(fract(uv) - 0.5);

    let r0 = _random_01(i + vec2<f32>(0.0, 0.0));
    let r1 = _random_01(i + vec2<f32>(1.0, 0.0));
    let r2 = _random_01(i + vec2<f32>(0.0, 1.0));
    let r3 = _random_01(i + vec2<f32>(1.0, 1.0));

    return mix(mix(r0, r1, f.x), mix(r2, r3, f.x), f.y);
}

fn builtin_simple_noise(uv: vec2<f32>, scale: f32) -> f32 {
    let t = 0.0;
    let t = t + _value_noise(vec2<f32>(uv.x * scale    , uv.y * scale    )) * 0.125;
    let t = t + _value_noise(vec2<f32>(uv.x * scale/2.0, uv.y * scale/2.0)) * 0.25;
    let t = t + _value_noise(vec2<f32>(uv.x * scale/4.0, uv.y * scale/4.0)) * 0.5;
    return t;
}


//  fn color_conv_rgb_rgb(input: vec3<f32>) -> vec3<f32> {
//      return input;
//  }
//  fn color_conv_linear_linear(input: vec3<f32>) -> vec3<f32> {
//      return input;
//  }
//  fn color_conv_hsv_hsv(input: vec3<f32>) -> vec3<f32> {
//      return input;
//  }


//  fn color_conv_rgb_linear(input: vec3<f32>) -> vec3<f32> {
//      let lo = input / 12.92;
//      let hi = pow(max(abs((input + 0.055) / 1.055), vec3<f32>(0.0000001192092896)), vec3<f32>(2.4));
//      return select(hi, lo, vec3<bool>(input.x <= 0.04045, input.y <= 0.04045, input.y <= 0.04045));
//  }

//  fn color_conv_rgb_hsv(input: vec3<f32>) -> vec3<f32> {
//      let k = vec4<f32>(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
//      let p = mix(vec4<f32>(input.bg, k.wz), vec4<f32>(input.gb, k.xy), step(input.b, input.g));
//      let q = mix(vec4<f32>(p.xyw, input.r), vec4<f32>(input.r, p.yzx), step(p.x, input.r));
//      let d = q.x - min(q.w, q.y);
//      let e = 0.0000000001;
//      return vec3<f32>(abs(q.z + (q.w - q.y)/(6.0 * d + e)), d / (q.x + e), q.x);
//  }

//  fn color_conv_linear_rgb(input: vec3<f32>) -> vec3<f32> {
//      let lo = input * 12.92;
//      let hi = (pow(max(abs(input), vec3<f32>(0.0000001192092896)), vec3<f32>(1.0 / 2.4, 1.0 / 2.4, 1.0 / 2.4)) * 1.055) - 0.055;
//      return select(hi, lo, vec3<bool>(input.x <= 0.0031308, input.y <= 0.0031308, input.y <= 0.0031308));
//  }

//  fn color_conv_linear_hls(input: vec3<f32>) -> vec3<f32> {
//      let lo = input * 12.92;
//      let hi = (pow(max(abs(input), vec3<f32>(0.0000001192092896)), vec3<f32>(1.0 / 2.4, 1.0 / 2.4, 1.0 / 2.4)) * 1.055) - 0.055;
//      let linear = select(hi, lo, vec3<bool>(input.x <= 0.0031308, input.y <= 0.0031308, input.y <= 0.0031308));
//      let k = vec4<f32>(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
//      let p = mix(vec4<f32>(linear.bg, k.wz), vec4<f32>(linear.gb, k.xy), step(linear.b, linear.g));
//      let q = mix(vec4<f32>(p.xyw, linear.r), vec4<f32>(linear.r, p.yzx), step(p.x, linear.r));
//      let d = q.x - min(q.w, q.y);
//      let e = 0.0000000001;
//      return vec3<f32>(abs(q.z + (q.w - q.y)/(6.0 * d + e)), d / (q.x + e), q.x);
//  }

//  fn color_conv_hsv_rgb(input: vec3<f32>) -> vec3<f32> {
//      let k = vec4<f32>(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
//      let p = abs(fract(input.xxx + k.xyz) * 6.0 - k.www);
//      return input.z * mix(k.xxx, clamp(p - k.xxx, vec3<f32>(0.0), vec3<f32>(1.0)), input.y);
//  }

//  fn color_conv_hsv_linear(input: vec3<f32>) -> vec3<f32> {
//      let k = vec4<f32>(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
//      let p = abs(fract(input.xxx + k.xyz) * 6.0 - k.www);
//      let rgb = input.z * mix(k.xxx, clamp(p - k.xxx, vec3<f32>(0.0), vec3<f32>(1.0)), input.y);
//      let lo = rgb / 12.92;
//      let hi = pow(max(abs((rgb + 0.055) / 1.055), vec3<f32>(1.192092896e-07)), vec3<f32>(2.4, 2.4, 2.4));
//      return select(hi, lo, vec3<bool>(input.x <= 0.04045, input.y <= 0.04045, input.y <= 0.04045));
//  }
