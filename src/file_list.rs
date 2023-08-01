use std::path::PathBuf;
use crate::widgets;



#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
struct FileListState {
    renaming: bool
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
    Delete(PathBuf),
    Rename(PathBuf, String),
    Select(usize),
    Deselect
}

pub struct FileListWidget<'a> {
    items: &'a Vec<PathBuf>,
    selected_item_index: Option<usize>
}

impl<'a> FileListWidget<'a> {
    pub fn new(items: &'a Vec<PathBuf>, selected_item_index: Option<usize>) -> Self {
        Self {
            items,
            selected_item_index
        }
    }

    pub fn show(&self, ui: &mut egui::Ui) -> Option<FileListAction> {
        let mut file_action = None;
        let width = ui.available_width();
        let height = self.file_item_height(ui);
        let total_rows = self.items.len();
        let id = egui::Id::new("file_list_state");
        let mut state = FileListState::load(ui.ctx(), id).unwrap_or(FileListState { renaming: false });
        let mut renaming = false;

        egui::ScrollArea::vertical().show_rows(ui, height, total_rows, |ui, row_range| {
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
        });
        
        state.renaming = renaming;
        state.store(ui.ctx(), id);

        file_action
    }

    fn file_item(&self, ui: &mut egui::Ui, path: &PathBuf, width: f32, selected: bool, renaming: bool) -> egui::InnerResponse<Option<String>> {
        let size = egui::vec2(width, self.file_item_height(ui));
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
                ui.add_space(4.0);
                let mut label = widgets::RenamableLabel::new(name);
                if renaming && selected {
                    label.rename();
                }
                label.show(ui)
            }).inner;
            egui::InnerResponse::new(text.inner, response)
        }).inner;
        response
    }

    fn file_item_height(&self, ui: &egui::Ui) -> f32 {
        let item_padding_y = 8.0;
        ui.spacing().interact_size.y + item_padding_y
    }
}