#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

pub mod core;

#[doc(inline)]
pub use crate::core::{Backend, Sink, Source, Console, Api};

#[cfg(feature = "wasmer")]
pub mod wasmer_backend;

#[cfg(feature = "wasmer")]
#[doc(inline)]
pub use crate::wasmer_backend::WasmerBackend;

#[cfg(feature = "wasmi")]
pub mod wasmi_backend;

#[cfg(feature = "wasmi")]
#[doc(inline)]
pub use crate::wasmi_backend::WasmiBackend;

#[cfg(feature = "sdl2-renderer")]
pub mod sdl2_renderer;

#[cfg(feature = "gpu-renderer")]
pub mod gpu_renderer;
