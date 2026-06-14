use crate::config;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;
use std::path::PathBuf;

const DEFAULT_AVATAR_FILE: &str = "default_avatar.png";

fn storage_dir() -> PathBuf {
    PathBuf::from(config::get_storage_dir())
}

pub fn save_png(name: &str, data: &[u8]) -> io::Result<String> {
    let dir = storage_dir();
    fs::create_dir_all(&dir)?;

    let filename = format!("{name}.png");
    let mut path = dir.clone();
    path.push(&filename);

    let mut file = File::create(&path)?;
    file.write_all(data)?;

    Ok(filename)
}

pub fn load_png(name: &str) -> Option<Vec<u8>> {
    let mut path = storage_dir();
    path.push(name);
    std::fs::read(path).ok()
}

pub fn load_avatar(name: &str) -> Option<Vec<u8>> {
    let filename = Path::new(name).file_name()?.to_owned();

    let mut path = storage_dir();
    path.push(filename);

    if let Ok(bytes) = std::fs::read(&path) {
        return Some(bytes);
    }

    let mut default_path = storage_dir();
    default_path.push(DEFAULT_AVATAR_FILE);
    std::fs::read(default_path).ok()
}
