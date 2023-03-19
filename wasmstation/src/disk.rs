//! Configuration for accessing game save files.

use std::{
    fs,
    io::{self, ErrorKind},
    path::PathBuf,
};

/// Common trait for accessing game disks.
pub trait DiskManager {
    /// Retrieve disk bytes from the system's store.
    fn read(&self) -> Result<[u8; 1024], String>;

    /// Write disk bytes to the system's store.
    fn write(&self, disk: [u8; 1024]) -> Result<(), String>;
}

/// A DiskManager which saves the disk at `/path/to/cart/{cart_name}.disk`.
pub struct Wasm4CompatibleDisk(PathBuf);

impl Wasm4CompatibleDisk {
    pub fn new(cart_location: &PathBuf) -> Self {
        let mut disk_location = cart_location.clone();
        disk_location.set_extension("disk");

        Self(disk_location)
    }
}

impl DiskManager for Wasm4CompatibleDisk {
    fn read(&self) -> Result<[u8; 1024], String> {
        let mut bytes: Vec<u8> = match fs::read(&self.0) {
            Ok(bytes) => Ok(bytes),
            Err(err) => match err.kind() {
                ErrorKind::NotFound => fs::write(&self.0, vec![0])
                    .map_err(|err| err.to_string())
                    .map(|_| vec![0]),
                _ => Err(err.to_string()),
            },
        }?;

        bytes.resize(1024, 0);
        bytes
            .try_into()
            .map_err(|_| "failed to resize save file to 1024".to_string())
    }

    fn write(&self, disk: [u8; 1024]) -> Result<(), String> {
        fs::write(&self.0, disk).map_err(|err| err.to_string())
    }
}

/// A `DiskManager` which saves the game disk at `$DATA_DIR/wasmstation/{name}.disk`.
///
/// |Platform | Location                                                                                         |
/// | ------- | ------------------------------------------------------------------------------------------------ |
/// | Linux   | `$XDG_DATA_HOME/wasmstation/{name}.disk` or `$HOME/.local/share/wasmstation/{name}.disk`         |
/// | macOS   | `$HOME/Library/Application Support/wasmstation/{name}.disk`                                      |
/// | Windows | `{FOLDERID_RoamingAppData}\wasmstation\{name}.disk`                                              |
pub struct UserwideDisk(PathBuf);

impl UserwideDisk {
    pub fn new(name: &str) -> Result<Self, io::Error> {
        let mut location = match dirs::data_dir() {
            Some(location) => location.join("wasmstation"),
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "no application data directory",
                ))
            }
        };

        fs::create_dir_all(&location)?;

        location.push(name);
        location.set_extension("disk");

        Ok(Self(location))
    }
}

impl DiskManager for UserwideDisk {
    fn read(&self) -> Result<[u8; 1024], String> {
        let mut bytes: Vec<u8> = match fs::read(&self.0) {
            Ok(bytes) => Ok(bytes),
            Err(err) => match err.kind() {
                ErrorKind::NotFound => fs::write(&self.0, vec![0])
                    .map_err(|err| err.to_string())
                    .map(|_| vec![0]),
                _ => Err(err.to_string()),
            },
        }?;

        bytes.resize(1024, 0);
        bytes
            .try_into()
            .map_err(|_| "failed to resize save file to 1024".to_string())
    }

    fn write(&self, disk: [u8; 1024]) -> Result<(), String> {
        fs::write(&self.0, disk).map_err(|err| err.to_string())
    }
}

/// The default disk manager, logs warning statements for every `read()`/`write()`.
///
/// This is created in order for [`LaunchConfig`](crate::renderer::LaunchConfig) to implement [`Default`].
pub struct DebugDisk;

impl DiskManager for DebugDisk {
    fn read(&self) -> Result<[u8; 1024], String> {
        log::warn!("DebugDisk used, no save disk read.");

        Ok((0..1024)
            .into_iter()
            .map(|_| 0)
            .collect::<Vec<u8>>()
            .try_into()
            .expect("wrong framebuffer size"))
    }

    fn write(&self, disk: [u8; 1024]) -> Result<(), String> {
        log::warn!("DebugDisk used, no save disk written.");

        Ok(())
    }
}
