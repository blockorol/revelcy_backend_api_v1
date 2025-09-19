use std::fs::{self, File};
use std::io::{self, Write};
use std::path::PathBuf;

const STORAGE_DIR: &str = "./storage";

pub fn save_png(name: &str, data: &[u8]) -> io::Result<String> {
    fs::create_dir_all(STORAGE_DIR)?; // создаём директорию, если нет

    let filename = format!("{name}.png");
    let mut path = PathBuf::from(STORAGE_DIR);
    path.push(&filename);

    let mut file = File::create(&path)?;
    file.write_all(data)?;

    Ok(filename)
}

pub fn load_png(name: &str) -> Option<Vec<u8>> {
    let mut path = PathBuf::from(STORAGE_DIR);
    path.push(name);
    std::fs::read(path).ok()
}
