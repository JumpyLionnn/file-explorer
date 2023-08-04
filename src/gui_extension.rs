use egui::InnerResponse;




pub trait UiHelpersExt {
    fn get_text_style_height(&self, style: egui::style::TextStyle) -> f32;
    fn calculate_rect_from_size(&self, size: egui::Vec2) -> egui::Rect;
    fn str_to_text_galley(&self, text: &str, fallback_font:  impl Into<egui::FontSelection>) -> egui::widget_text::WidgetTextGalley;
    fn custom_widget<T>(&mut self, size: egui::Vec2, add_contents: impl FnOnce(&mut egui::Ui, egui::Rect, &mut egui::Response) -> T) -> InnerResponse<T>;

    fn pointer_pressed_at(&self, rect: egui::Rect) -> bool;
    fn pointer_pressed_outside_of(&self, rect: egui::Rect) -> bool;
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

    fn custom_widget<T>(&mut self, size: egui::Vec2, add_contents: impl FnOnce(&mut egui::Ui, egui::Rect, &mut egui::Response) -> T) -> InnerResponse<T> {
        let (rect, mut response) = self.allocate_at_least(size, egui::Sense::click_and_drag());
        // hacky but there isnt any other way to allocate a rect and add widgets on top of it
        // move the cursor up the height
        // TODO: change that
        self.add_space(-(rect.height() + self.spacing().item_spacing.y));
        let mut inner_res = self.allocate_ui_at_rect(rect, |ui| {
            add_contents(ui, rect, &mut response)
        });
        inner_res.response = response;
        inner_res
    }

    fn pointer_pressed_at(&self, rect: egui::Rect) -> bool {
        if self.input(|input| input.pointer.any_pressed()) {
            self.rect_contains_pointer(rect)
        }
        else {
            false
        }
    }
    fn pointer_pressed_outside_of(&self, rect: egui::Rect) -> bool {
        if self.input(|input| input.pointer.any_pressed()) {
            !self.rect_contains_pointer(rect)
        }
        else {
            false
        }
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