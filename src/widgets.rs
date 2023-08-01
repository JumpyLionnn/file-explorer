use std::path::{PathBuf, self};

use crate::gui_extension::*;




pub fn path_navigation_bar(ui: &mut egui::Ui, path: &PathBuf, width: f32) -> Option<path::PathBuf> {
    let component_padding = egui::vec2(5.0, 7.0);
    let total_component_padding = component_padding * 2.0;
    let height = ui.get_text_style_height(egui::style::TextStyle::Button) + component_padding.y * 2.0 ;
    let size = egui::vec2(width, height);
    let rect = ui.calculate_rect_from_size(size);
    let visuals = ui.visuals();
    ui.painter().rect_stroke(rect, egui::Rounding::none(), visuals.window_stroke);

    let mut path_component_clicked = None;

    ui.horizontal(|ui| {
        ui.style_mut().spacing.item_spacing.x = 0.0;
        let mut it = path.iter().peekable();
        let mut component_path = path::PathBuf::new();
        while let Some(component) = it.next() {
            let str = component.to_string_lossy();
            let text = str.trim_matches(|c| c == '/' || c == '\\');
            
            if !text.is_empty() {
                // TODO: make a separate path struct
                let str = component_path.as_mut_os_string();
                str.push(component);
                str.push("/");
                let show_arrow_head = it.peek().is_some();
                let text = ui.str_to_text_galley(text, egui::TextStyle::Button);
                let text_size = text.size();
                let mut button_arrow_spacing = 0.0;
                let mut arrow_head_width = 0.0;
                if show_arrow_head {
                    button_arrow_spacing = total_component_padding.x;
                    arrow_head_width = text_size.y / 3.6;
                }
                let desired_size = text_size + total_component_padding + egui::vec2(button_arrow_spacing + arrow_head_width, 0.0);
                
                let (rect, response) = ui.allocate_at_least(desired_size, egui::Sense::click());
                response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, text.text()));
                if response.clicked() {
                    path_component_clicked = Some(component_path.clone());
                }

                if ui.is_rect_visible(rect) {
                    let visuals = ui.style().interact(&response);
                    let (text_rect, arrow_head_rect) = split_rect_horizontally(rect, button_arrow_spacing, component_padding, text_size.x, arrow_head_width);
                    if response.hovered() {
                        ui.painter().rect(rect, egui::Rounding::none(), visuals.weak_bg_fill, egui::Stroke::NONE);
                    }
                    text.paint_with_visuals(ui.painter(), text_rect.min, visuals);

                    if show_arrow_head {
                        let arrow_head_size = egui::vec2(arrow_head_width, arrow_head_width * 2.0);
                        let arrow_head_pos = ui.layout().align_size_within_rect(arrow_head_size, arrow_head_rect).min;
                        let stoke = egui::Stroke::new(2.0, visuals.fg_stroke.color);
                        ui.painter().arrow_head(arrow_head_pos, arrow_head_size, stoke);
                    }
                }
            }
        }
    });

    path_component_clicked
}

pub fn error_dialog(ctx: &egui::Context, message: &str) -> bool {
    let mut open = true;
    let center = ctx.screen_rect().center();
    let res = egui::Window::new("Error")
        .collapsible(false)
        .resizable(false)
        .open(&mut open)
        .default_pos(center)
        .pivot(egui::Align2::CENTER_CENTER)
        .show(ctx, |ui|{
            ui.label(message);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                ui.style_mut().spacing.button_padding = (24.0, 4.0).into();
                if ui.button("ok").clicked() {
                    return true;
                }
                false
            }).inner
        });
    if !open {
        return true;
    }
    if let Some(res) = res {
        return res.inner.unwrap_or(false);
    }
    return false;
}

pub fn delete_dialog(ctx: &egui::Context, name: &str, item_type: &str) -> Option<bool> {
    let mut open = true;
    let center = ctx.screen_rect().center();
    let res = egui::Window::new(format!("Delete {item_type}"))
        .collapsible(false)
        .resizable(false)
        .open(&mut open)
        .default_pos(center)
        .pivot(egui::Align2::CENTER_CENTER)
        .show(ctx, |ui|{
            ui.label(format!("Are you sure you want to delete this {item_type}?"));
            ui.label(name);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                ui.style_mut().spacing.button_padding = (24.0, 4.0).into();
                if ui.button("No").clicked() {
                    return Some(false);
                }
                if ui.button("Yes").clicked() {
                    return Some(true);
                }
                None
            }).inner
        });
    if !open {
        return Some(false);
    }
    if let Some(res) = res {
        return res.inner.unwrap_or(None);
    }
    return None;
}


#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
struct RenamableLabelState {
    text: Option<String>
}
impl RenamableLabelState {
    pub fn load(ctx: &egui::Context, id: egui::Id) -> Option<Self> {
        ctx.data(|d| d.get_temp(id))
    }

    pub fn store(self, ctx: &egui::Context, id: egui::Id) {
        ctx.data_mut(|d| d.insert_temp(id, self))
    }
}

pub struct RenamableLabel {
    text: String,
    request_renaming: bool
}

const RENAMABLE_LABLE_INPUT_MARGIN: egui::Vec2 = egui::vec2(4.0, 2.0);

impl RenamableLabel {
    pub fn new(text: String) -> Self {
        Self { 
            text,
            request_renaming: false
        }
    }

    pub fn rename(&mut self) -> &mut Self {
        self.request_renaming = true;
        self
    }

    pub fn show(self, ui: &mut egui::Ui) -> egui::InnerResponse<Option<String>> {
        let id = egui::Id::new(&self.text);
        let mut state = RenamableLabelState::load(ui.ctx(), id).unwrap_or(RenamableLabelState { text: None });
        match state.text {
            Some(mut text) => {
                let (renamed, response) = input(ui, &mut text, false);
                let res;
                if renamed {
                    res = egui::InnerResponse::new(if text.is_empty() { None } else { Some(text) }, response);
                    state.text = None;
                    state.store(ui.ctx(), id);
                }
                else {
                    res = egui::InnerResponse::new(None, response);
                    state.text = Some(text);
                    state.store(ui.ctx(), id);
                }
                res
            },
            None => {
                if self.request_renaming {
                    let mut text = self.text;
                    let (renamed, response) = input(ui, &mut text, true);
                    if renamed {
                        let state = RenamableLabelState {
                            text: None
                        };
                        state.store(ui.ctx(), id);
                        egui::InnerResponse::new(if text.is_empty() { None } else { Some(text) }, response)
                    }
                    else {
                        let state = RenamableLabelState {
                            text: Some(text)
                        };
                        state.store(ui.ctx(), id);
                        egui::InnerResponse::new(None, response)  
                    }
                }
                else {
                    ui.horizontal(|ui| {
                        ui.add_space(RENAMABLE_LABLE_INPUT_MARGIN.x);
                        egui::InnerResponse::new(None, ui.label(self.text))
                    }).inner
                }
            }
        }
    }
}

fn input(ui: &mut egui::Ui, text: &mut String, select: bool) -> (bool, egui::Response) {
    let text_edit = egui::TextEdit::singleline(text).margin(RENAMABLE_LABLE_INPUT_MARGIN);
    let mut output = text_edit.show(ui);
    if select {
        output.response.request_focus();
        let selection = egui::text_edit::CCursorRange::two(
            egui::text::CCursor::new(0), 
            egui::text::CCursor::new(text.len())
        );
        output.state.set_ccursor_range(Some(selection));
        output.state.store(ui.ctx(), output.response.id);
    }
    
    (output.response.lost_focus(), output.response)
}