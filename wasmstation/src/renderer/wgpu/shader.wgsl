@group(0) @binding(0) var<uniform> palette: mat4x4<f32>;
@group(0) @binding(1) var<uniform> window_size: vec2<u32>;
@group(0) @binding(2) var framebuffer: texture_1d<u32>;

@vertex
fn vs_main(@location(0) position: vec3<f32>) -> @builtin(position) vec4<f32> {
    return vec4<f32>(position, 1.0);
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    // physical side length of the screen
    let p: u32 = min(window_size.x, window_size.y);
    // sizes of the letterbox borders
    let b: vec2<u32> = (window_size - p) / 2u;

    // "physical" size on screen
    let pu: vec2<u32> = vec2<u32>(u32(position.x), u32(position.y));

    // return black if we're outside the screen
    if (!all(pu >= b && pu <= (window_size - b))) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }

    // wasm4 framebuffer index and position
    let fb_pos: vec2<f32> = vec2<f32>((pu - b)) * f32(p / 160u);
    let fb_idx: u32 = (u32(fb_pos.y) * 160u) + u32(fb_pos.x);

    // return color in palette from pixel
    return palette[(textureLoad(framebuffer, i32(fb_idx / 4u), 0i)[0] >> ((fb_idx % 4u) * 2u)) & 0x3u];
}