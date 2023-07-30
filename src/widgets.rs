use std::path::{PathBuf, self};
use crate::gui_extension::*;

pub fn file_item(ui: &mut egui::Ui, entry: &PathBuf, width: f32, selected: bool) -> egui::Response {
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