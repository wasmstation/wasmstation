@group(0) @binding(0) var<uniform> palette: mat4x4<f32>;
@group(0) @binding(1) var<uniform> window_size: vec2<u32>;
@group(0) @binding(2) var framebuffer: texture_1d<u32>;

@vertex
fn vs_main(@location(0) position: vec3<f32>) -> @builtin(position) vec4<f32> {
    return vec4<f32>(position, 1.0);
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let p: f32 = min(f32(window_size.x), f32(window_size.y)); // physical side length of the game screen (largest square possible square)
    let b: vec2<u32> = (window_size - u32(p)) / 2u; // sizes of the "letterbox" borders
    let pu: vec2<u32> = vec2<u32>(u32(position.x), u32(position.y)); // "physical" location on screen in uint format

    if (!all(pu >= b && pu <= (window_size - b))) {
        // return black if we're outside the screen
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }

    let fpos: vec2<f32> = vec2<f32>(pu - b) * (1.0 / (p / 160.0)); // window coords -> game screen coords
    let fidx: u32 = (u32(fpos.y) * 160u) + u32(fpos.x); // index into framebuffer from coords

    return palette[(textureLoad(framebuffer, i32(fidx / 4u), 0i)[0] >> ((fidx % 4u) * 2u)) & 0x3u];
}
