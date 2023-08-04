use std::path::PathBuf;
use std::sync::mpsc;

use notify::RecursiveMode;
use notify::Watcher as NotitifyWatcher;

#[derive(Debug)]
pub enum Change {
    Unknown,
    Create(CreateKind, PathBuf),
    Remove(PathBuf),
    Rename(PathBuf, PathBuf),
    Modify(PathBuf)
}

#[derive(Debug)]
pub enum CreateKind {
    File,
    Directory
}
pub trait Watcher {
    fn watch(&mut self, path: PathBuf) -> Result<(), String>;
    fn look_for_changes(&self) -> Option<Change>;
}

pub struct FileSystemWatcher {
    children_watcher: notify::RecommendedWatcher,
    receiver: mpsc::Receiver<Change>,
    current_path: PathBuf
}

impl FileSystemWatcher {
    pub fn new<F>(path: PathBuf, on_change: F) -> Self
        where F: Fn() + Send + 'static {
        let (sender, receiver) = mpsc::channel();
        let mut expect_rename_to = None;
        let mut watcher = notify::recommended_watcher(move |event: Result<notify::Event, notify::Error>| {
            let changes = if let Ok(event) = event {
                get_notify_changes(event, &mut expect_rename_to)
            }
            else {
                // error needs to be handled somehow
                // TODO: is a rescan needed? 
                Vec::new()
            };
            for change in changes {
                if let Ok(()) = sender.send(change) {
                    on_change();
                }
            }
        }).unwrap();
        watcher.watch(path.as_path(), RecursiveMode::NonRecursive).unwrap();

        Self { children_watcher: watcher, receiver, current_path: path }
    }
}

impl Watcher for FileSystemWatcher {
    fn watch(&mut self, path: PathBuf) -> Result<(), String> {
        let _ = self.children_watcher.unwatch(&self.current_path);
        if let Err(error) = self.children_watcher.watch(&path, notify::RecursiveMode::NonRecursive) {
            let error_message = match error.kind {
                notify::ErrorKind::Generic(message) => message,
                notify::ErrorKind::Io(error) => error.to_string(),
                other => format!("Internal error {:?}", other)
            };
            return Err(error_message);
        }
        else {
            Ok(())
        }
    }

    fn look_for_changes(&self) -> Option<Change> {
        self.receiver.try_recv().ok()
    }
}

fn get_notify_changes(mut event: notify::Event, expect_rename_to: &mut Option<PathBuf>) -> Vec<Change> {
    if event.need_rescan() {
        vec![Change::Unknown]
    }
    else {
        match event.kind {
            notify::EventKind::Any => vec![Change::Unknown],
            notify::EventKind::Create(notify::event::CreateKind::File) => {
                let mut creations = Vec::new();
                for path in event.paths {
                    creations.push(Change::Create(CreateKind::File, path));
                }
                creations
            },
            notify::EventKind::Create(notify::event::CreateKind::Folder) => {
                let mut creations = Vec::new();
                for path in event.paths {
                    creations.push(Change::Create(CreateKind::Directory, path));
                }
                creations
            },
            notify::EventKind::Create(notify::event::CreateKind::Any | notify::event::CreateKind::Other) => {
                let mut creations = Vec::new();
                for path in event.paths {
                    if path.is_dir() {
                        creations.push(Change::Create(CreateKind::Directory, path));
                    }
                    else if path.is_file() {
                        creations.push(Change::Create(CreateKind::File, path));
                    }
                }
                creations
            },
            notify::EventKind::Modify(notify::event::ModifyKind::Data(notify::event::DataChange::Size)) => {
                let mut resizes = Vec::new();
                for path in event.paths {
                    resizes.push(Change::Modify(path));
                }
                resizes
            },
            notify::EventKind::Modify(notify::event::ModifyKind::Any) |
            notify::EventKind::Modify(notify::event::ModifyKind::Data(_)) |
            notify::EventKind::Modify(notify::event::ModifyKind::Metadata(_)) => {
                let mut metadatas = Vec::new();
                for path in event.paths {
                    metadatas.push(Change::Modify(path));
                }
                metadatas
            },
            notify::EventKind::Modify(notify::event::ModifyKind::Name(notify::event::RenameMode::From)) => {
                assert!(expect_rename_to.is_none());
                assert!(event.paths.len() == 1);
                *expect_rename_to = Some(event.paths.pop().expect("rename from even should have 1 path"));
                vec![]
            },
            notify::EventKind::Modify(notify::event::ModifyKind::Name(notify::event::RenameMode::To)) => {
                assert!(expect_rename_to.is_some());
                assert!(event.paths.len() == 1);
                vec![Change::Rename(expect_rename_to.take().expect("There is no previous path from"), event.paths.pop().expect("rename mode both should have 1 path"))]
            },
            notify::EventKind::Modify(notify::event::ModifyKind::Name(notify::event::RenameMode::Both)) => {
                assert!(event.paths.len() == 2);
                let to = event.paths.pop().expect("rename mode both should have 2 paths");
                vec![Change::Rename(event.paths.pop().expect("rename mode both should have 2 paths"), to)]
            },
            notify::EventKind::Remove(_) => {
                let mut removals = Vec::new();
                for path in event.paths {
                    removals.push(Change::Remove(path));
                }
                removals
            },
            _other => {
                Vec::new()
            }
        }
    }
}