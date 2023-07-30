


pub trait UiHelpersExt {
    fn get_text_style_height(&self, style: egui::style::TextStyle) -> f32;
    fn calculate_rect_from_size(&self, size: egui::Vec2) -> egui::Rect;
    fn str_to_text_galley(&self, text: &str, fallback_font:  impl Into<egui::FontSelection>) -> egui::widget_text::WidgetTextGalley;
}

impl UiHelpersExt for egui::Ui {
    fn get_text_style_height(&self, style: egui::style::TextStyle) -> f32 {
        let font_id = style.resolve(self.style());
        self.fonts(|f| f.row_height(&font_id))
    }

    fn calculate_rect_from_size(&self, size: egui::Vec2) -> egui::Rect {
        let top_left = self.cursor().min;
        egui::Rect::from_min_size(top_left, size)
    }

    fn str_to_text_galley(&self, text: &str, fallback_font:  impl Into<egui::FontSelection>) -> egui::widget_text::WidgetTextGalley {
        let text: egui::WidgetText = text.into();
        text.into_galley(self, Some(false), f32::INFINITY, fallback_font)
    }
}

pub trait PainterPrimitiveExt {
    fn arrow_head(&self, position: egui::Pos2, size: egui::Vec2, stroke: egui::Stroke);
}

impl PainterPrimitiveExt for egui::Painter {
    fn arrow_head(&self, position: egui::Pos2, size: egui::Vec2, stroke: egui::Stroke) {
        let tip = egui::pos2(position.x + size.x, position.y + size.y / 2.0);
        let bottom_left = egui::pos2(position.x, position.y + size.y);
        self.line_segment([position, tip], stroke);
        self.line_segment([bottom_left, tip], stroke);
    }
}

pub fn split_rect_horizontally(parent: egui::Rect, spacing: f32, padding: egui::Vec2, a_width: f32, b_width: f32) -> (egui::Rect, egui::Rect) {
    let content_rect = parent.shrink2(padding);
    let a = egui::Rect::from_min_size(content_rect.min, egui::vec2(a_width, content_rect.height()));
    let b_x = content_rect.min.x + a_width + spacing;
    let b = egui::Rect::from_min_size(egui::pos2(b_x, content_rect.min.y), egui::vec2(b_width, content_rect.height()));
    return (a, b);
}