use std::path::PathBuf;

use std::collections::HashMap;
use crate::{widgets, gui_extension::UiHelpersExt};



#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
struct FileListState {
    renaming: bool,
    new_item: Option<NewItem>
}

#[derive(Clone)]
pub struct NewItem {
    pub kind: ItemKind,
    pub name: String
}

#[derive(Clone)]
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

pub struct FileIconManager {
    icons: HashMap<String, egui_extras::RetainedImage>,
    directory_icon: egui_extras::RetainedImage
}

fn get_file_extension(name: &str) -> &str {
    match name.rfind('.') {
        Some(idk) => &name[idk..name.len()],
        None => "unknown",
    }
}

impl FileIconManager {
    pub fn new() -> Self {
        let buffer = include_bytes!("../assets/folder-icon.png");
        let image = egui_extras::RetainedImage::from_image_bytes("assets/folder-icon.png", buffer)
            .expect("unable to read the folder icon image");
        Self {
            icons: HashMap::new(),
            directory_icon: image
        }
    }

    pub fn get_icon(&mut self, name: &str) -> Option<&egui_extras::RetainedImage> {
        let extension = get_file_extension(name);
        Some(match self.icons.entry(extension.to_owned()) {
            std::collections::hash_map::Entry::Occupied(entry) => entry.into_mut(),
            std::collections::hash_map::Entry::Vacant(entry) => {
                let bytes: Vec<u8> = systemicons::get_icon(&format!(".{extension}"), 16).ok()?;
                let image = egui_extras::RetainedImage::from_image_bytes(extension, &bytes).ok()?;
                entry.insert(image)
            },
        })
    }

    pub fn get_directory_icon(&self) -> &egui_extras::RetainedImage {
        &self.directory_icon
    }
}

impl FileListState {
    pub fn load(ctx: &egui::Context, id: egui::Id) -> Option<Self> {
        ctx.data(|d| d.get_temp(id))
    }

    pub fn store(self, ctx: &egui::Context, id: egui::Id) {
        ctx.data_mut(|d| d.insert_temp(id, self))
    }
}
pub enum FileListAction {
    Open(PathBuf),
    Create(NewItem),
    Delete(PathBuf),
    Rename(PathBuf, String),
    Select(usize),
    Deselect
}

pub struct FileListWidget<'a> {
    items: &'a Vec<PathBuf>,
    selected_item_index: Option<usize>,
    icons: &'a mut FileIconManager,
    new_item: Option<ItemKind>
}
const FILE_ITEM_PADDING: f32 = 4.0;
impl<'a> FileListWidget<'a> {
    pub fn new(items: &'a Vec<PathBuf>, selected_item_index: Option<usize>, icons: &'a mut FileIconManager) -> Self {
        Self {
            items,
            selected_item_index,
            icons,
            new_item: None
        }
    }

    pub fn new_item(&mut self, item_kind: ItemKind) {
        self.new_item = Some(item_kind);
    }

    pub fn show(&mut self, ui: &mut egui::Ui) -> Option<FileListAction> {
        let mut file_action = None;
        let width = ui.available_width();
        let height = self.file_item_total_height(ui);
        let id = egui::Id::new("file_list_state");
        let mut state = FileListState::load(ui.ctx(), id).unwrap_or(FileListState { renaming: false, new_item: None });
        if let Some(kind) = self.new_item.take() {
            state.new_item = Some(NewItem { kind, name: String::new() });
        }
        let total_rows = self.items.len() + if state.new_item.is_some() {1} else {0};
        let mut renaming = false;

        egui::ScrollArea::vertical().show_rows(ui, height, total_rows, |ui, mut row_range| {
            if let Some(mut new_item) = state.new_item.take() {
                row_range.end -= 1;
                let item = self.temp_file_item(ui, &mut new_item, width);
                if item.inner {
                    if !new_item.name.is_empty() && !ui.input(|input| input.key_pressed(egui::Key::Escape)) {
                        file_action = Some(FileListAction::Create(new_item));
                    }
                }
                else {
                    state.new_item = Some(new_item);
                }
            }
            
            for index in row_range {
                let entry = &self.items[index];
                let selected = match self.selected_item_index {
                    Some(selected_index) => selected_index == index,
                    None => false,
                };
                let item = self.file_item(ui, entry, width, selected, state.renaming);
                
                if item.response.double_clicked() {
                    file_action = Some(FileListAction::Open(entry.clone()));
                }
                else if item.response.is_pointer_button_down_on() { // TODO: this can also trigger when clicking any other mouse button, open an issue
                    file_action = Some(FileListAction::Select(index));
                }
                else if selected && item.response.clicked_elsewhere() {
                    file_action = Some(FileListAction::Deselect);
                }
                if let Some(text) = item.inner {
                    file_action = Some(FileListAction::Rename(entry.clone(), text));
                }
                
                item.response.context_menu(|ui| {
                    if ui.button("delete").clicked() {
                        file_action = Some(FileListAction::Delete(entry.clone()));
                        ui.close_menu();
                    }
                    if ui.button("rename").clicked() {
                        file_action = None;
                        renaming = true;
                        ui.close_menu();
                    }
                });
            }
        }).inner_rect;  
        state.renaming = renaming;
        state.store(ui.ctx(), id);

        file_action
    }

    fn file_item(&mut self, ui: &mut egui::Ui, path: &PathBuf, width: f32, selected: bool, renaming: bool) -> egui::InnerResponse<Option<String>> {
        let size = egui::vec2(width, self.file_item_total_height(ui));
        let name =  path.file_name().unwrap_or_else(|| std::ffi::OsStr::new("unknown")).to_string_lossy().to_string();
        
        let response = ui.push_id(egui::Id::new(&name), |ui| {
            let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
            response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, &name));
            if ui.is_rect_visible(rect) {
                let painter = ui.painter();
                let style = ui.style();
                let visuals = style.interact_selectable(&response, selected);
                if response.hovered() || selected {
                    painter.rect(rect, egui::Rounding::none(), visuals.bg_fill, egui::Stroke::NONE);
                }
            }
            // hacky but there isnt any other way to allocate a rect and add widgets on top of it
            // move the cursor up the height
            // TODO: change that
            ui.add_space(-(rect.height() + ui.spacing().item_spacing.y)); 
    
            let text = ui.allocate_ui_with_layout(size, egui::Layout::left_to_right(egui::Align::Center), |ui| {
                ui.spacing_mut().item_spacing = egui::vec2(FILE_ITEM_PADDING, 0.0);
                ui.add_space(FILE_ITEM_PADDING);
                let ctx = ui.ctx().clone();
                let mut label = widgets::RenamableLabel::new(name, &ctx);
                self.add_icon(ui, &ItemKind::from_path(path), label.get_text());
                if renaming && selected {
                    label.rename();
                }
                label.show(ui)
            }).inner;
            egui::InnerResponse::new(text.inner, response)
        }).inner;
        response
    }

    fn temp_file_item(&mut self, ui: &mut egui::Ui, item: &mut NewItem, width: f32) -> egui::InnerResponse<bool> {
        let size = egui::vec2(width, self.file_item_total_height(ui));
        let response = ui.push_id(egui::Id::new("temp_file_item"), |ui| {
            let rect = ui.calculate_rect_from_size(size);
            if ui.is_rect_visible(rect) {
                ui.painter().rect(rect, egui::Rounding::none(), ui.visuals().selection.bg_fill, egui::Stroke::NONE);
            }
            ui.allocate_ui_with_layout(size, egui::Layout::left_to_right(egui::Align::Center), |ui| {
                ui.add_space(4.0);
                self.add_icon(ui, &item.kind, &item.name);
                let respnonse = ui.text_edit_singleline(&mut item.name);
                respnonse.lost_focus()
            }).inner
        });
        response
    }

    fn add_icon(&mut self, ui: &mut egui::Ui, kind: &ItemKind, name: &str) {
        let height = self.file_item_height(ui);
        let icon_size = egui::vec2(height, height);
        match kind {
            ItemKind::File => {
                if let Some(icon) = self.icons.get_icon(&name) {
                    icon.show_size(ui, icon_size);
                }
            },
            ItemKind::Directory => {
                self.icons.get_directory_icon().show_size(ui, icon_size);
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