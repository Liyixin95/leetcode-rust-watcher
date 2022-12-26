use crossbeam::channel::Receiver;
use notify::event::{CreateKind, RemoveKind};
use notify::{Event as NotifyEvent, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub enum Event {
    Create(PathBuf),
    Delete(PathBuf),
}

impl Event {
    pub(crate) fn path(&self) -> &Path {
        match &self {
            Event::Create(path) => path,
            Event::Delete(path) => path,
        }
    }
}

pub fn watch(path: &Path) -> anyhow::Result<(RecommendedWatcher, Receiver<Event>)> {
    let (tx, rx) = crossbeam::channel::unbounded();

    let mut watcher = notify::recommended_watcher(move |res: notify::Result<NotifyEvent>| {
        let ev = match res {
            Ok(ev) => ev,
            Err(e) => {
                log::error!("watch fail, cause: {e}");
                return;
            }
        };

        let f = match ev.kind {
            EventKind::Create(CreateKind::File | CreateKind::Any) => Event::Create,
            EventKind::Remove(RemoveKind::File | RemoveKind::Any) => Event::Delete,
            ev => {
                log::debug!("other {ev:?}");
                return;
            }
        };

        ev.paths
            .into_iter()
            .map(|path| tx.send(f(path)))
            .find_map(|res| res.err())
            .map(|err| log::error!("send fail, {err}"));
    })?;

    watcher.watch(path, RecursiveMode::NonRecursive)?;

    Ok((watcher, rx))
}
