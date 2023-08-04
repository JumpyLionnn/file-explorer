use std::collections::HashMap;

pub struct IconManager {
    icons: HashMap<String, egui_extras::RetainedImage>,
    directory_icon: egui_extras::RetainedImage
}

impl IconManager {
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

fn get_file_extension(name: &str) -> &str {
    match name.rfind('.') {
        Some(idk) => &name[idk..name.len()],
        None => "unknown",
    }
}
