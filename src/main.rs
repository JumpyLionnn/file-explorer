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
    receiver: mpsc::Receiver<()>,
    selected_index: Option<usize>,
    error_dialog: Option<String>
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
            receiver,
            selected_index: None,
            error_dialog: None
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


    fn open(&mut self, path: PathBuf) -> Result<(), String> {
        if path.is_dir() {
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
        }
        else {
            if let Err(err) = open::that(path) {
                let error_message = format!("{err}");
                return Err(error_message);
            }
        }
        Ok(())
    }

    fn file_list(&mut self, ui: &mut egui::Ui) -> Option<PathBuf> {
        let mut open_request = None;
        let width = ui.min_rect().width();
        for (index, entry) in self.child_directories.iter().enumerate() {
            let selected = match self.selected_index {
                Some(selected_index) => selected_index == index,
                None => false,
            };
            let item = file_item(ui, entry, width, selected);
            if item.double_clicked() {
                open_request = Some(entry.clone());
            }
            else if item.is_pointer_button_down_on() {
                self.selected_index = Some(index);
            }
            else if selected && item.clicked_elsewhere() {
                self.selected_index = None;
            }
        }
        open_request
    }
}

impl eframe::App for FileExplorer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Ok(_) = self.receiver.try_recv() {
            self.refresh_childs();
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(self.directory.to_str().expect("Non utf-8 directory name"));
            
            let open_request = self.file_list(ui);
            if let Some(path) = open_request {
                if let Err(error) = self.open(path) {
                    self.error_dialog = Some(error);
                }
            }
        });

        if self.error_dialog.is_some() {
            let mut open = true;
            let center = ctx.screen_rect().center();
            egui::Window::new("Error")
                .collapsible(false)
                .resizable(false)
                .open(&mut open)
                .default_pos(center)
                .pivot(egui::Align2::CENTER_CENTER)
                .show(ctx, |ui|{
                    let message: &String = &self.error_dialog.as_ref().unwrap();
                    ui.label(message);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                        ui.style_mut().spacing.button_padding = (24.0, 4.0).into();
                        if ui.button("ok").clicked() {
                            self.error_dialog = None;
                        }
                    });
                });
            if !open {
                self.error_dialog = None;
            }
        }

        
    }
}

fn file_item(ui: &mut egui::Ui, entry: &PathBuf, width: f32, selected: bool) -> egui::Response {
    let height = ui.spacing().interact_size.y;
    let padding = 4.0;
    let size = egui::vec2(width, height + padding);
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let style = ui.style();
        let font_id = egui::style::TextStyle::Button.resolve(style);
        let row_height = ui.fonts(|f| f.row_height(&font_id)) + ui.spacing().item_spacing.y;
        let visuals = style.interact_selectable(&response, selected);
        if response.hovered() || selected {
            painter.rect(rect, egui::Rounding::none(), visuals.bg_fill, egui::Stroke::NONE);
        }
        let name = match entry.file_name() {
            Some(name) => name.to_str().unwrap_or("Unable to display the name"),
            None => "Unable to display the name",
        };
        let text_left_margin = 5.0;
        let text_pos = egui::Pos2 {
            x: rect.min.x + text_left_margin,
            y: rect.min.y + row_height / 2.0 + padding / 2.0
        };
        painter.text(text_pos, egui::Align2::LEFT_CENTER, name, font_id, visuals.fg_stroke.color);
    }

    response
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


