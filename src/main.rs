use eframe::egui;

fn main() {
    eframe::run_native("file explorer", Default::default(), Box::new(|_cc| Box::<FileExplorer>::default())).unwrap();
}

struct FileExplorer {

}

impl Default for FileExplorer {
    fn default() -> Self {
        Self {  }
    }
}

impl eframe::App for FileExplorer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My file explorer");
            ui.label("Hello world!");
        });
    }
}