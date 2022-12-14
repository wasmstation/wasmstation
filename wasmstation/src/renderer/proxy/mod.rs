use crate::Renderer;

use super::{WgpuRenderer, Sdl2Renderer};


pub enum ProxyRenderer {
    #[cfg(feature="wgpu-renderer")]
    Wgpu(WgpuRenderer),
    #[cfg(feature="sdl2-renderer")]
    Sdl2(Sdl2Renderer),
}

const PROXY_RENDERER_NAMES: &[&str] = &["wgpu", "sdl2"];
impl ProxyRenderer {

    pub fn from_name(name: &str, display_scale: u32) -> Result<ProxyRenderer,()> {
        match name {
            "wgpu" => Ok(ProxyRenderer::Wgpu(WgpuRenderer{display_scale})),
            "sdl2" => Ok(ProxyRenderer::Sdl2(Sdl2Renderer{})),
            _ => Err(())
        }
    }
    pub fn names() -> &'static [&'static str] {
        return PROXY_RENDERER_NAMES;

    }
}

impl Renderer for ProxyRenderer {
    fn present(self, b: impl crate::Backend + 'static) {
        match self {
            #[cfg(feature="wgpu-renderer")]
            Self::Wgpu(r) => r.present(b),
            #[cfg(feature="sdl2-renderer")]
            Self::Sdl2(r) => r.present(b),
        }
    }
}