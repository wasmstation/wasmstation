cfg_if::cfg_if! {
    if #[cfg(all(feature = "wgpu-renderer", feature = "sdl2-renderer"))] {
        compile_error!("renderer cannot be both wgpu and sdl2");
    } else if #[cfg(feature = "wgpu-renderer")] {
        mod wgpu;
        pub use self::wgpu::WgpuRenderer;

        pub type DefaultRenderer = WgpuRenderer;
    } else if #[cfg(feature = "sdl2-renderer")] {
        mod sdl2;
        pub use self::sdl2::Sdl2Renderer;

        pub type DefaultRenderer = Sdl2Renderer;
    }
}
