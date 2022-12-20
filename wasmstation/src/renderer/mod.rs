#[cfg(feature = "wgpu-renderer")]
mod wgpu;

#[cfg(feature = "wgpu-renderer")]
pub use self::wgpu::WgpuRenderer;

#[cfg(feature = "embedded")]
mod embedded;

#[cfg(feature = "embedded")]
pub use {self::embedded::draw, embedded_graphics};

#[cfg(feature = "embedded-renderer")]
mod embedded_renderer;

#[cfg(feature = "embedded-renderer")]
pub use self::embedded_renderer::EmbeddedRendererSimulator;

mod proxy;
pub use proxy::ProxyRenderer;
