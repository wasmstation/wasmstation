#[cfg(feature = "wgpu-renderer")]
mod wgpu;

#[cfg(feature = "wgpu-renderer")]
pub use self::wgpu::WgpuRenderer;

#[cfg(feature = "sdl2-renderer")]
mod sdl2;

#[cfg(feature = "sdl2-renderer")]
pub use self::sdl2::Sdl2Renderer;

mod proxy;
pub use proxy::ProxyRenderer;
