use std::fs::{self, File};
use std::io::{self, Write};
use std::path::PathBuf;

const STORAGE_DIR: &str = "./storage";
fn storage_dir() -> PathBuf {
    match env::var("STORAGE_DIR") {
        Ok(path) => PathBuf::from(path),
        Err(_) => {
            log::error!("env STORAGE_DIR not found, setted ./storage");
            PathBuf::from("./storage")
        }
    }
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
