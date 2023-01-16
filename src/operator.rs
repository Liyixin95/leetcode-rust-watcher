use std::fs::OpenOptions;
use std::io;
use std::io::{Read, Write};
use std::path::PathBuf;
use crossbeam::channel::{Receiver, TryRecvError};
use crate::mapping::Mapping;
use crate::watcher::Event;

pub struct Operator {
    rx: Receiver<Event>,
    mapping: Mapping,
    lib_path: PathBuf,
    dir_path: PathBuf,
}

impl Operator {
    pub fn new(dir_path: PathBuf, rx: Receiver<Event>) -> anyhow::Result<Self> {
        let mut lib_path = dir_path.clone();
        lib_path.push("lib.rs");
        let mut lib_file = OpenOptions::new().read(true).open(&lib_path)?;

        let mut lib_content = String::new();
        lib_file.read_to_string(&mut lib_content)?;
        Ok(Self {
            rx,
            mapping: Mapping::from_str(&lib_content)?,
            lib_path,
            dir_path,
        })
    }

    pub fn run(mut self) -> anyhow::Result<()> {
        loop {
            loop {
                match self.rx.try_recv() {
                    Ok(ev) => self.handle_event(ev),
                    Err(TryRecvError::Empty) => break,
                    Err(e @ TryRecvError::Disconnected) => return Err(e.into()),
                }
            }

            self.mapping.cleanup(&self.dir_path);
            self.flush()?;

            let ev = self.rx.recv()?;
            self.handle_event(ev);
        }
    }

    fn handle_event(&mut self, ev: Event) {
        let Some(file_name) = ev.path().file_name() else {
            log::error!("invalid path : {}", ev.path().display());
            return
        };

        let Some(file_name) = file_name.to_str()  else {
            log::error!("invalid coding: {}", file_name.to_string_lossy());
            return
        };

        let filter = ["main.rs", "Cargo.toml", "Cargo.lock", "target"];

        if filter.contains(&file_name) {
            return;
        }

        match &ev {
            Event::Create(path) => {
                if let Err(e) = append_use(path) {
                    log::error!("fail to appen use statement in {}, cause: {e}", path.display());
                }

                self
                    .mapping
                    .insert_leetcode_file(file_name)
                    .map(|_| log::info!("create file:  {}", path.display()))
                    .unwrap_or_else(|e| log::error!("{e}"))
            },
            Event::Delete(_) => self
                .mapping
                .delete_file(file_name)
                .map(|removed| log::info!("remove {removed} from mapping"))
                .unwrap_or_else(|| log::warn!("{file_name} not exists in mapping")),
        };
    }

    fn flush(&self) -> anyhow::Result<()> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .truncate(true)
            .create(true)
            .open(&self.lib_path)?;

        file.write_all(self.mapping.to_string().as_bytes())?;

        Ok(())
    }
}

fn append_use(path: &PathBuf) -> io::Result<()> {
    let mut file = OpenOptions::new().read(true).write(true).open(path)?;
    let mut buf = String::from("use crate::data_struct::*;\n");
    file.read_to_string(&mut buf)?;
    file.write_all(buf.as_bytes())?;
    file.set_len(buf.len() as u64)?;
    file.flush()
}
