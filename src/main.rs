mod widgets;
mod gui_extension;
mod file_list;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::mpsc;
use notify::Watcher;
use eframe::egui;

fn main() {
    eframe::run_native("file explorer", Default::default(), Box::new(|cc| Box::new(FileExplorer::new(cc)))).unwrap();
}
struct FileExplorer {
    child_directories: Vec<PathBuf>,
    directory: PathBuf,
    watcher: notify::RecommendedWatcher,
    receiver: mpsc::Receiver<()>,
    selected_index: Option<usize>,
    error_dialog: Option<String>,
    delete_dialog: Option<PathBuf>
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

        let mut style = (*cc.egui_ctx.style()).clone();
        style.visuals.menu_rounding = egui::Rounding::none();
        style.spacing.menu_margin = egui::Margin::same(2.0);
        cc.egui_ctx.set_style(style);

        let mut explorer =  Self { 
            directory: current_dir,
            child_directories: Vec::new(),
            watcher,
            receiver,
            selected_index: None,
            error_dialog: None,
            delete_dialog: None
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

    fn change_dir(&mut self, path: PathBuf) -> Result<(), String> {
        let _ = self.watcher.unwatch(&self.directory.as_path());
        if let Err(err) = self.watcher.watch(path.as_path(), notify::RecursiveMode::NonRecursive) {
            let error_message = match err.kind {
                notify::ErrorKind::Generic(message) => message,
                notify::ErrorKind::Io(error) => format!("{error}"),
                other => format!("Internal error {:?}", other)
            };
            return Err(error_message);
        }
        self.directory = path;
        self.refresh_childs();
        Ok(())
    }

    fn open(&mut self, path: PathBuf) -> Result<(), String> {
        if path.is_dir() {
            self.change_dir(path)?;
        }
        else {
            if let Err(err) = open::that(path) {
                let error_message = format!("{err}");
                return Err(error_message);
            }
        }
        Ok(())
    }

    fn try_delete_item(&mut self, path: PathBuf) -> Result<(), std::io::Error> {
        if path.is_file() {
            if let Ok(metadata) = fs::metadata(&path) {
                if metadata.len() > 0 {
                    self.delete_dialog = Some(path);
                }
                else {
                    fs::remove_file(&path)?;
                }
            }
        }
        else if path.is_dir() {
            if fs::read_dir(&path)?.next().is_some() {
                self.delete_dialog = Some(path);
            }
            else {
                fs::remove_dir(&path)?;
            }
        }
        Ok(())
    }
}

impl eframe::App for FileExplorer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Ok(_) = self.receiver.try_recv() {
            self.refresh_childs();
        }
        
        egui::CentralPanel::default().show(ctx, |ui| {
            let width = ui.available_width();
            if let Some(path) = widgets::path_navigation_bar(ui, &self.directory, width) {
                if let Err(message) = self.change_dir(path) {
                    self.error_dialog = Some(message);
                }
            }
            let mut file_list = file_list::FileListWidget::new(&self.child_directories, self.selected_index);
            ui.horizontal(|ui| {
                if ui.button("new folder").clicked() {
                    file_list.new_item(file_list::ItemKind::Directory);
                }
                if ui.button("new file").clicked() {
                    file_list.new_item(file_list::ItemKind::File);
                }
            });
            let open_request = file_list.show(ui);
            if let Some(action) = open_request {
                match action {
                    file_list::FileListAction::Open(path) => {
                        if let Err(error) = self.open(path) {
                            self.error_dialog = Some(error);
                        }
                    },
                    file_list::FileListAction::Create(item) => {
                        match item.kind {
                            file_list::ItemKind::File => {
                                if let Err(error) = fs::write(item.name, "") {
                                    self.error_dialog = Some(error.to_string());
                                }
                            },
                            file_list::ItemKind::Directory => {
                                if let Err(error) = fs::create_dir(item.name) {
                                    self.error_dialog = Some(error.to_string());
                                }
                            },
                        }
                    }, 
                    file_list::FileListAction::Delete(path) => {
                        if let Err(error) = self.try_delete_item(path) {
                            self.error_dialog = Some(error.to_string());
                        }
                    },
                    file_list::FileListAction::Rename(path, name) => {
                        if let Err(error) = fs::rename(path, name) {
                            self.error_dialog = Some(error.to_string());
                        }
                    },
                    file_list::FileListAction::Select(index) => {
                        self.selected_index = Some(index);
                    },
                    file_list::FileListAction::Deselect => {
                        self.selected_index = None;
                    },
                }
            } 
        });

        if let Some(message) = &self.error_dialog {
            if widgets::error_dialog(ctx, message) {
                self.error_dialog = None;
            }
        }
        if let Some(path) = &self.delete_dialog {
            let name = path.file_name().unwrap_or_else(|| std::ffi::OsStr::new("unknown")).to_string_lossy().to_string();
            let item_type = if path.is_file() {"file"} else if path.is_dir() {"folder"} else {"item"};
            if let Some(res) = widgets::delete_dialog(ctx, &name, item_type) {
                if res {
                    if path.is_file() {
                        if let Err(err) = fs::remove_file(path) {
                            self.error_dialog = Some(err.to_string());
                        }
                    }
                    else if path.is_dir() {
                        if let Err(err) = fs::remove_dir_all(path) {
                            self.error_dialog = Some(err.to_string());
                        }
                    }
                }
                self.delete_dialog = None;
            }
        }
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