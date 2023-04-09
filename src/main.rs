#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // Hide console window on Windows in release
#![cfg(all(target_arch = "x86_64", target_os = "windows"))] // Set target os as Windows

mod highligher;
mod log;
mod settings;
mod ui;

use ui::CodeEditor;

fn main() {
    let Some(editor) = CodeEditor::new() else {
        // Settings could not be loaded
        return;
    };

    if cfg!(target_os = "windows") {
        eframe::run_native(
            "Colors",
            eframe::NativeOptions {
                icon_data: load_image("images\\coding.png"),
                maximized: true,
                ..Default::default()
            },
            Box::new(|_cc| Box::new(editor)),
        )
    }
}

/// Load an image using the [`image`] crate. Return None if the image could not be opened
fn load_image(path: &str) -> Option<eframe::IconData> {
    let Some(img) = image::open(path).ok() else {
        log::warning("App image could not be found");
        return None;
    };

    let img = img.into_rgba8();
    let (width, height) = img.dimensions();
    let rgba = img.into_raw();
    Some(eframe::IconData {
        rgba,
        width,
        height,
    })
}
