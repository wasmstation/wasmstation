use std::{fs, io::ErrorKind, path::PathBuf};

/// A simple reusable closure for writing a game disk to a file
pub fn write(save_path: &PathBuf) -> impl Fn([u8; 1024]) -> Result<(), String> + '_ {
    move |data: [u8; 1024]| fs::write(save_path, data).map_err(|err| err.to_string())
}

/// A simple reusable closure for reading a game disk from a file.
pub fn read(save_path: &PathBuf) -> impl Fn() -> Result<[u8; 1024], String> + '_ {
    move || {
        // read bytes from disk and create new blank file if it doesn't exist.
        let mut bytes: Vec<u8> = match fs::read(&save_path) {
            Ok(bytes) => Ok(bytes),
            Err(err) => match err.kind() {
                ErrorKind::NotFound => fs::write(&save_path, vec![0])
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
}
