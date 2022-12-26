use crate::mapping::Mapping;
use crate::watcher::Event;
use crossbeam::channel::{Receiver, TryRecvError};
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::PathBuf;

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
        let mut lib_file = OpenOptions::new().read(true).create(true).open(&lib_path)?;

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
            log::error!("{} is not file", ev.path().display());
            return
        };

        if file_name == "main.rs" {
            log::info!("create main file");
            return;
        }

        match &ev {
            Event::Create(path) => self
                .mapping
                .insert_file(path.clone(), file_name)
                .map(|_| log::info!("create file:  {}", path.display()))
                .unwrap_or_else(|e| log::error!("{e}")),
            Event::Delete(_) => self
                .mapping
                .delete_file(file_name)
                .map(|removed| log::info!("remove {} from mapping", removed.to_string_lossy()))
                .unwrap_or_else(|| {
                    log::warn!("{} not exists in mapping", file_name.to_string_lossy())
                }),
        };
    }

    fn flush(&self) -> anyhow::Result<()> {
        let content = self.mapping.print();
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .truncate(true)
            .create(true)
            .open(&self.lib_path)?;

        file.write_all(content.as_bytes())?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use os_str_bytes::OsStrBytes;
    use proc_macro2::Literal;

    fn accept_str(_input: &str) {}

    #[test]
    fn test1() {
        let s = String::new();
        let refs = &s;
        accept_str(refs);
    }

    #[test]
    fn test() {
        let os_string = OsString::from("test");
        let lit = Literal::byte_string(os_string.as_os_str().to_raw_bytes().as_ref());

        eprintln!("lit = {:#?}", lit);
    }
}
