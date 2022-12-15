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
    pub fn from_name(name: &str, display_scale: u32) -> Result<ProxyRenderer, ()> {
        match name {
            #[cfg(feature = "wgpu-renderer")]
            "wgpu" => Ok(ProxyRenderer::Wgpu(WgpuRenderer { display_scale })),
            #[cfg(feature = "sdl2-renderer")]
            "sdl2" => Ok(ProxyRenderer::Sdl2(Sdl2Renderer {})),
            _ => Err(()),
        }
    }

    pub fn default_name() -> &'static str {
        ProxyRenderer::names()[0]
    }

    pub fn names() -> &'static [&'static str] {
        PROXY_RENDERER_NAMES
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
