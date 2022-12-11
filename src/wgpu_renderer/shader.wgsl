@group(0) @binding(0) var<uniform> palette: mat4x4<f32>;
@group(0) @binding(1) var<uniform> display_scale: u32;
@group(0) @binding(2) var framebuffer: texture_1d<u32>;

@vertex
fn vs_main(@location(0) position: vec3<f32>) -> @builtin(position) vec4<f32> {
    return vec4<f32>(position, 1.0);
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let adj_position = vec2<f32>(position.x / f32(display_scale), position.y / f32(display_scale));
    let idx: u32 = (u32(adj_position.y) * 160u) + u32(adj_position.x);

    // TODO: make the window look ok when scaling
    
    return palette[(textureLoad(framebuffer, i32(idx / 4u), 0i)[0] >> ((idx % 4u) * 2u)) & 0x3u];
}