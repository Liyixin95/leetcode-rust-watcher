use std::fs::{read_dir, OpenOptions};
use std::io;
use std::io::Write;
use std::path::{Path, PathBuf};
use crate::mapping::Mapping;
use anyhow::anyhow;

const DATA_STRUCT_FILE: &'static str = "data_struct.rs";

pub fn init_workspace(dir_path: PathBuf) -> anyhow::Result<()> {
    let mut data_struct_path = dir_path.clone();
    data_struct_path.push(DATA_STRUCT_FILE);
    create_file(data_struct_path, data_struct_rs())?;

    let mut cargo_toml_path = dir_path.clone();
    cargo_toml_path.push("Cargo.toml");
    let cur_dir_name = dir_path
        .file_name()
        .ok_or_else(|| anyhow!("dir name not found"))?
        .to_string_lossy();
    create_file(cargo_toml_path, cargo_toml(&cur_dir_name))?;

    let content = lib_rs(&dir_path)?;
    let mut lib_path = PathBuf::from(dir_path);
    lib_path.push("lib.rs");
    create_file(lib_path, content)?;
    Ok(())
}

fn create_file(path: PathBuf, content: String) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(path)?;

    file.write_all(content.as_bytes())
}

fn data_struct_rs() -> String {
    const DATA_STRUCT: &str = include_str!("./data_struct.rs.temp");
    DATA_STRUCT.to_string()
}

fn lib_rs(dir: &Path) -> io::Result<String> {
    let dir_iter = read_dir(dir)?;
    let mut mapping = Mapping::default();
    for ret in dir_iter {
        let entry = ret?;
        let Ok(typ) = entry.file_type() else {
            continue;
        };

        if !typ.is_file() {
            continue;
        };

        let file_name = entry.file_name();
        let Some(file_name) = file_name.to_str() else {
            log::error!("invalid coding: {}", entry.path().display());
            continue;
        };

        if file_name == DATA_STRUCT_FILE {
            mapping.insert_file(file_name);
        } else {
            let _ = mapping.insert_leetcode_file(file_name);
        }
    }

    Ok(mapping.to_string())
}

fn cargo_toml(name: &str) -> String {
    format!(
        r#"
    [package]
    name = "{name}"
    version = "0.1.0"
    edition = "2021"

    [lib]
    path = "lib.rs"

    # See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html

    [dependencies]
    "#
    )
}
