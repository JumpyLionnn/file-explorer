use eframe::egui;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::mpsc;
use notify::Watcher;

fn main() {
    eframe::run_native("file explorer", Default::default(), Box::new(|cc| Box::new(FileExplorer::new(cc)))).unwrap();
}
struct FileExplorer {
    child_directories: Vec<PathBuf>,
    directory: PathBuf,
    watcher: notify::RecommendedWatcher,
    receiver: mpsc::Receiver<()>
}

impl FileExplorer {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let current_dir = env::current_dir().expect("Couldnt get the working directory!");
        let (sender, receiver) = mpsc::channel();
        let context = cc.egui_ctx.clone();
        let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
            if let Ok(event) = res {
                if should_refresh_dir(event) {
                    sender.send(()).unwrap();
                    context.request_repaint();
                }
            }
        }).unwrap();
        watcher.watch(&current_dir.as_path(), notify::RecursiveMode::NonRecursive).unwrap();

        let mut explorer =  Self { 
            directory: current_dir,
            child_directories: Vec::new(),
            watcher,
            receiver
        };

        explorer.refresh_childs();

        explorer
    }

    fn refresh_childs(&mut self) {
        self.child_directories.clear();
        if let Ok(entries) = fs::read_dir(&self.directory) {
            for entry in entries {
                if let Ok(entry) = entry {
                    self.child_directories.push(entry.path());
                }
            }
        }
    }
}

impl eframe::App for FileExplorer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Ok(_) = self.receiver.try_recv() {
            self.refresh_childs();
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(self.directory.to_str().expect("Non utf-8 directory name"));
            let entries = &*self.child_directories;
            for entry in entries {
                let name = entry.file_name();
                if let Some(name) = name {
                    ui.label(name.to_str().expect("The file name is not a valid utf-8"));
                }
            }
        });
    }
}

fn should_refresh_dir(change: notify::Event) -> bool {
    // Its hard to use the changes api notify provides so for now there will be just a refreshed when any changes are made
    match change.kind {
        notify::event::EventKind::Create(_) | 
        notify::event::EventKind::Remove(_)  => true,
        notify::event::EventKind::Modify(kind) => match kind {
            notify::event::ModifyKind::Name(rename_kind) => match rename_kind {
                notify::event::RenameMode::From => false,
                _other => true
            },
            _other => false
        },
        _other => false
    }
}


