mod widgets;
mod gui_extension;
mod file_list;
mod watcher;
mod icon_manager;
use std::env;
use std::fs;
use std::path::PathBuf;
use eframe::egui;
use file_list::FileListItem;
use watcher::Watcher;

fn main() {
    eframe::run_native("file explorer", Default::default(), Box::new(|cc| Box::new(FileExplorer::new(cc)))).unwrap();
}
struct FileExplorer {
    child_directories: Vec<file_list::FileListItem>,
    directory: PathBuf,
    error_dialog: Option<String>,
    delete_dialog: Option<PathBuf>,
    file_icons_manager: icon_manager::IconManager,
    watcher: Box<dyn Watcher>,
    file_list: file_list::FileListWidget
}

impl FileExplorer {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let current_dir = env::current_dir().expect("Couldnt get the working directory!");

        let mut style = (*cc.egui_ctx.style()).clone();
        style.visuals.menu_rounding = egui::Rounding::none();
        style.spacing.menu_margin = egui::Margin::same(2.0);
        cc.egui_ctx.set_style(style);

        let context = cc.egui_ctx.clone();
        let watcher = Box::new(watcher::FileSystemWatcher::new(current_dir.clone(), move || {
            context.request_repaint();
        }));

        let mut explorer =  Self { 
            directory: current_dir,
            child_directories: Vec::new(),
            error_dialog: None,
            delete_dialog: None,
            file_icons_manager: icon_manager::IconManager::new(),
            watcher: watcher,
            file_list: file_list::FileListWidget::new()
        };

        explorer.refresh_childs();

        explorer
    }

    fn refresh_childs(&mut self) {
        self.child_directories.clear();
        if let Ok(entries) = fs::read_dir(&self.directory) {
            for entry in entries {
                if let Ok(entry) = entry {
                    self.child_directories.push(FileListItem::new(entry.path()));
                }
            }
        }
    }

    fn change_dir(&mut self, path: PathBuf) -> Result<(), String> {
        self.watcher.watch(path.clone())?;
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
        if let Some(change) = self.watcher.look_for_changes() {
            match change {
                watcher::Change::Unknown => self.refresh_childs(),
                watcher::Change::Create(_kind, path) => self.child_directories.push(FileListItem::new(path)),
                watcher::Change::Remove(path) => self.child_directories.retain(|p| *p.path != path),
                watcher::Change::Rename(from, to) => {
                    let item = self.child_directories.iter_mut().find(|p| *p.path == from);
                    if let Some(item) = item {
                        *item = FileListItem::new(to);
                    }
                },
                watcher::Change::Modify(_) => {},
            }
        }
        
        egui::CentralPanel::default().show(ctx, |ui| {
            let width = ui.available_width();
            if let Some(path) = widgets::path_navigation_bar(ui, &self.directory, width) {
                if let Err(message) = self.change_dir(path) {
                    self.error_dialog = Some(message);
                }
            }
            ui.horizontal(|ui| {
                if ui.button("new folder").clicked() {
                    self.file_list.new_item(file_list::ItemKind::Directory);
                }
                if ui.button("new file").clicked() {
                    self.file_list.new_item(file_list::ItemKind::File);
                }
            });
            let actions = self.file_list.show(ui, &self.child_directories, &mut self.file_icons_manager);
            for action in actions {
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
                        self.child_directories[index].selected = true;
                    },
                    file_list::FileListAction::Deselect(index) => {
                        self.child_directories[index].selected = false;
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