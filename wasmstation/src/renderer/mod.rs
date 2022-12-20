#[cfg(feature = "embedded")]
mod embedded;

#[cfg(feature = "embedded")]
pub use self::embedded::draw;

#[cfg(feature = "cpu-renderer")]
mod cpu;

#[cfg(feature = "cpu-renderer")]
pub use self::cpu::CpuRenderer;

#[cfg(feature = "gpu-renderer")]
mod gpu;

#[cfg(feature = "gpu-renderer")]
pub use self::gpu::GpuRenderer;

mod proxy;
pub use proxy::ProxyRenderer;
