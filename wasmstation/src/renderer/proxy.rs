use std::str::FromStr;

use crate::Renderer;

#[cfg(feature = "embedded-renderer")]
use super::EmbeddedRendererSimulator;
#[cfg(feature = "wgpu-renderer")]
use super::WgpuRenderer;

pub enum ProxyRenderer {
    #[cfg(feature = "wgpu-renderer")]
    Wgpu(WgpuRenderer),
    #[cfg(feature = "embedded-renderer")]
    Embedded(EmbeddedRendererSimulator),
}

const AVAILABLE_RENDERERS: &[&str] = &[
    #[cfg(feature = "wgpu-renderer")]
    "wgpu",
    #[cfg(feature = "embedded-renderer")]
    "embedded",
];

impl ProxyRenderer {
    pub fn available_renderers() -> &'static [&'static str] {
        AVAILABLE_RENDERERS
    }

    pub fn set_display_scale(&mut self, scale: u32) {
        match self {
            #[cfg(feature = "wgpu-renderer")]
            ProxyRenderer::Wgpu(r) => r.display_scale = scale,
            #[cfg(feature = "embedded-renderer")]
            ProxyRenderer::Embedded(r) => r.display_scale = scale,
        }
    }

    pub fn set_title(&mut self, title: impl AsRef<str>) {
        match self {
            #[cfg(feature = "wgpu-renderer")]
            ProxyRenderer::Wgpu(r) => r.title = title.as_ref().to_string(),
            #[cfg(feature = "embedded-renderer")]
            ProxyRenderer::Embedded(r) => r.title = title.as_ref().to_string(),
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
            #[cfg(feature = "wgpu-renderer")]
            "wgpu" => Ok(ProxyRenderer::Wgpu(WgpuRenderer::default())),
            #[cfg(feature = "embedded-renderer")]
            "embedded" => Ok(ProxyRenderer::Embedded(EmbeddedRendererSimulator::default())),
            _ => Err("Unrecognized renderer".to_string()),
        }
    }
}

impl Renderer for ProxyRenderer {
    fn present(self, b: impl crate::Backend + 'static) {
        match self {
            #[cfg(feature = "wgpu-renderer")]
            Self::Wgpu(r) => r.present(b),
            #[cfg(feature = "embedded-renderer")]
            Self::Embedded(r) => r.present(b),
        }
    }
}
