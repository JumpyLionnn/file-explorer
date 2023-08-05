use std::path::PathBuf;

use crate::{widgets, gui_extension::UiHelpersExt};
use crate::icon_manager::IconManager;

pub struct NewItem {
    pub kind: ItemKind,
    pub name: String
}

pub enum ItemKind {
    File,
    Directory
}

impl ItemKind {
    pub fn from_path(path: &PathBuf) -> Self {
        if path.is_dir() {
            Self::Directory
        }
        else {
            Self::File
        }
    }
}

pub enum FileListAction {
    Open(PathBuf),
    Create(NewItem),
    Delete(PathBuf),
    Rename(PathBuf, String),
    Select(usize),
    Deselect(usize)
}

pub struct FileListItem {
    pub path: PathBuf,
    pub selected: bool
}

impl FileListItem {
    pub fn new(path: PathBuf) -> Self {
        Self { path: path, selected: false }
    }
}

pub struct FileListWidget {
    new_item: Option<NewItem>,
    rename_request: bool
}
const FILE_ITEM_PADDING: f32 = 4.0;
impl FileListWidget {
    pub fn new() -> Self {
        Self {
            new_item: None,
            rename_request: false
        }
    }

    pub fn new_item(&mut self, item_kind: ItemKind) {
        self.new_item = Some(NewItem { kind: item_kind, name: String::new() });
    }

    pub fn show(&mut self, ui: &mut egui::Ui, items: &Vec<FileListItem>, icons: &mut IconManager) -> Vec<FileListAction> {
        let mut actions= Vec::new();
        let width = ui.available_width();
        let height = self.file_item_total_height(ui);
        let total_rows = items.len() + if self.new_item.is_some() {1} else {0};
        let renaming = self.rename_request;
        if self.rename_request {
            self.rename_request = false;
        }

        egui::ScrollArea::vertical().show_rows(ui, height, total_rows, |ui, mut row_range| {
            if let Some(mut new_item) = self.new_item.take() {
                row_range.end -= 1;
                let item = self.temp_file_item(ui, &mut new_item, width, icons);
                if item.inner {
                    if !new_item.name.is_empty() && !ui.input(|input| input.key_pressed(egui::Key::Escape)) {
                        actions.push(FileListAction::Create(new_item));
                    }
                }
                else {
                    self.new_item = Some(new_item);
                }
            }
            
            for index in row_range {
                let item = &items[index];
                let item_response = self.file_item(ui, item, width, renaming, icons);
                
                let rect = item_response.response.rect;
                if item_response.response.double_clicked() {
                    actions.push(FileListAction::Open(item.path.clone()));
                }
                else if ui.pointer_pressed_at(rect) && ui.is_enabled() {
                    actions.push(FileListAction::Select(index));
                }
                if let Some(text) = item_response.inner {
                    actions.push(FileListAction::Rename(item.path.clone(), text));
                }
                
                let pressed_outside = ui.pointer_pressed_outside_of(rect);
                let mut context_menu_clicked = false;
                item_response.response.context_menu(|ui| {
                    if ui.button("delete").clicked() {
                        actions.push(FileListAction::Delete(item.path.clone()));
                        ui.close_menu();
                    }
                    if ui.button("rename").clicked() {
                        self.rename_request = true;
                        ui.close_menu();
                    }
                    context_menu_clicked = ui.pointer_pressed_at(ui.max_rect());
                });
                if item.selected && pressed_outside && !context_menu_clicked && ui.is_enabled() {
                    actions.push(FileListAction::Deselect(index));
                }
            }
        }).inner_rect;  
        actions
    }

    fn file_item(&mut self, ui: &mut egui::Ui, item: &FileListItem, width: f32, renaming: bool, icons: &mut IconManager) -> egui::InnerResponse<Option<String>> {
        let size = egui::vec2(width, self.file_item_total_height(ui));
        let name =  item.path.file_name().unwrap_or_else(|| std::ffi::OsStr::new("unknown")).to_string_lossy().to_string();
        let response = ui.custom_widget(size, |ui, rect, response| {
            response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, &name));
            if ui.is_rect_visible(rect) {
                let visuals = ui.style().interact_selectable(&response, item.selected);
                if response.hovered() || item.selected {
                    ui.painter().rect(rect, egui::Rounding::none(), visuals.bg_fill, egui::Stroke::NONE);
                }
            }

            let text = ui.horizontal_centered(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(FILE_ITEM_PADDING, 0.0);
                ui.add_space(FILE_ITEM_PADDING);
                let ctx = ui.ctx().clone();
                let mut label = widgets::RenamableLabel::new(name, &ctx);
                self.add_icon(ui, &ItemKind::from_path(&item.path), label.get_text(), icons);
                if renaming && item.selected {
                    label.rename();
                }
                label.show(ui).inner
            }).inner;
            text
        });
        response
    }

    fn temp_file_item(&mut self, ui: &mut egui::Ui, item: &mut NewItem, width: f32, icons: &mut IconManager) -> egui::InnerResponse<bool> {
        let size = egui::vec2(width, self.file_item_total_height(ui));
        let response = ui.push_id(egui::Id::new("temp_file_item"), |ui| {
            let rect = ui.calculate_rect_from_size(size);
            if ui.is_rect_visible(rect) {
                ui.painter().rect(rect, egui::Rounding::none(), ui.visuals().selection.bg_fill, egui::Stroke::NONE);
            }
            ui.allocate_ui_with_layout(size, egui::Layout::left_to_right(egui::Align::Center), |ui| {
                ui.add_space(4.0);
                self.add_icon(ui, &item.kind, &item.name, icons);
                let respnonse = ui.text_edit_singleline(&mut item.name);
                respnonse.lost_focus()
            }).inner
        });
        response
    }

    fn add_icon(&mut self, ui: &mut egui::Ui, kind: &ItemKind, name: &str, icons: &mut IconManager) {
        let height = self.file_item_height(ui);
        let icon_size = egui::vec2(height, height);
        match kind {
            ItemKind::File => {
                if let Some(icon) = icons.get_icon(&name) {
                    ui.image_consider_disabled(icon, icon_size);
                }
            },
            ItemKind::Directory => {
                ui.image_consider_disabled(icons.get_directory_icon(), icon_size);
            },
        }
    }

    fn file_item_total_height(&self, ui: &egui::Ui) -> f32 {
        self.file_item_height(ui) + FILE_ITEM_PADDING * 2.0
    }

    fn file_item_height(&self, ui: &egui::Ui) -> f32 {
        ui.spacing().interact_size.y
    }
    
}