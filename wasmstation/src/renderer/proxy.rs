use std::str::FromStr;

use crate::Renderer;

#[cfg(feature = "sdl2-renderer")]
use super::Sdl2Renderer;
#[cfg(feature = "wgpu-renderer")]
use super::WgpuRenderer;

pub enum ProxyRenderer {
    #[cfg(feature = "wgpu-renderer")]
    Wgpu(WgpuRenderer),
    #[cfg(feature = "sdl2-renderer")]
    Sdl2(Sdl2Renderer),
}

const PROXY_RENDERER_NAMES: &[&str] = &[
    #[cfg(feature = "wgpu-renderer")]
    "wgpu",
    #[cfg(feature = "sdl2-renderer")]
    "sdl2",
];

impl ProxyRenderer {
    pub fn possible_names() -> &'static [&'static str] {
        PROXY_RENDERER_NAMES
    }

    pub fn set_display_scale(&mut self, scale: u32) {
        match self {
            ProxyRenderer::Wgpu(w) => w.display_scale = scale,
            _ => (),
        }
    }
}

impl Default for ProxyRenderer {
    fn default() -> Self {
        Self::from_str(PROXY_RENDERER_NAMES[0]).unwrap()
    }
}

impl FromStr for ProxyRenderer {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            #[cfg(feature = "wgpu-renderer")]
            "wgpu" => Ok(ProxyRenderer::Wgpu(WgpuRenderer::default())),
            #[cfg(feature = "sdl2-renderer")]
            "sdl2" => Ok(ProxyRenderer::Sdl2(Sdl2Renderer::default())),
            _ => Err("Unrecognized renderer".to_string()),
        }
    }
}

impl Renderer for ProxyRenderer {
    fn present(self, b: impl crate::Backend + 'static) {
        match self {
            #[cfg(feature = "wgpu-renderer")]
            Self::Wgpu(r) => r.present(b),
            #[cfg(feature = "sdl2-renderer")]
            Self::Sdl2(r) => r.present(b),
        }
    }
}
