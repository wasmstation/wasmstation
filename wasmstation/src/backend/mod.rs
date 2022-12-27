#[cfg(feature = "wasmer-backend")]
mod wasmer;

#[cfg(feature = "wasmer-backend")]
pub use self::wasmer::WasmerBackend;
