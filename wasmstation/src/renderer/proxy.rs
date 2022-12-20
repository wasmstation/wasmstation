use std::str::FromStr;

use crate::Renderer;

#[cfg(feature = "cpu-renderer")]
use super::CpuRenderer;
#[cfg(feature = "gpu-renderer")]
use super::GpuRenderer;

pub enum ProxyRenderer {
    #[cfg(feature = "gpu-renderer")]
    Gpu(GpuRenderer),
    #[cfg(feature = "cpu-renderer")]
    Cpu(CpuRenderer),
}

const AVAILABLE_RENDERERS: &[&str] = &[
    #[cfg(feature = "gpu-renderer")]
    "gpu",
    #[cfg(feature = "cpu-renderer")]
    "cpu",
];

impl ProxyRenderer {
    pub fn available_renderers() -> &'static [&'static str] {
        AVAILABLE_RENDERERS
    }

    pub fn set_display_scale(&mut self, scale: u32) {
        match self {
            #[cfg(feature = "gpu-renderer")]
            ProxyRenderer::Gpu(r) => r.display_scale = scale,
            #[cfg(feature = "cpu-renderer")]
            ProxyRenderer::Cpu(r) => r.display_scale = scale,
        }
    }

    pub fn set_title(&mut self, title: impl AsRef<str>) {
        match self {
            #[cfg(feature = "gpu-renderer")]
            ProxyRenderer::Gpu(r) => r.title = title.as_ref().to_string(),
            #[cfg(feature = "cpu-renderer")]
            ProxyRenderer::Cpu(r) => r.title = title.as_ref().to_string(),
        }
    }
}

impl Default for ProxyRenderer {
    fn default() -> Self {
        Self::from_str(AVAILABLE_RENDERERS[0]).unwrap()
    }
}

impl FromStr for ProxyRenderer {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            #[cfg(feature = "gpu-renderer")]
            "gpu" => Ok(ProxyRenderer::Gpu(GpuRenderer::default())),
            #[cfg(feature = "cpu-renderer")]
            "cpu" => Ok(ProxyRenderer::Cpu(CpuRenderer::default())),
            _ => Err("Unrecognized renderer".to_string()),
        }
    }
}

impl Renderer for ProxyRenderer {
    fn present(self, b: impl crate::Backend + 'static) {
        match self {
            #[cfg(feature = "gpu-renderer")]
            Self::Gpu(r) => r.present(b),
            #[cfg(feature = "cpu-renderer")]
            Self::Cpu(r) => r.present(b),
        }
    }
}
