///! Importing user settings from `settings.json`.
///! The following `struct`s represent the deserialization of the JSON file into
///! Rust elements.
use std::fs;

use serde_derive::Deserialize;

use super::log;

const SETTINGS_PATH: &str = "settings\\settings.json";

/// Representation of the color of code elements in the editor. Colors are
/// defined as arrays of three [`u8`], as per RGB standard.
#[derive(Deserialize, Clone, Copy)]
pub struct CodeColor {
    pub ident: [u8; 3],
    pub number: [u8; 3],
    pub string: [u8; 3],
    pub symbol: [u8; 3],
    pub keyword: [u8; 3],
    pub builtin_fn: [u8; 3],
    pub fun: [u8; 4],
    pub comment: [u8; 3],
    pub error: [u8; 3],
    pub other: [u8; 3],
}

/// Represent the whole file `settings.json`
#[derive(Deserialize)]
pub struct Settings {
    pub code_color: CodeColor,
    pub save_btn: bool,      // enable the save button?
    pub save_and_run: bool,  // save the file before running it?
    pub save_on_close: bool, // save the current file before closing the IDE?
    pub code_font_size: f32,
    pub console_font_size: f32,
    pub betty_exe_path: String,
}

/// Try to retrieve the JSON contents in the settings file, and try to deserialize
/// the data as a [`Settings`] `struct`. If it is not possible to do any of them,
/// log the error and return [`None`].
impl Settings {
    pub fn get() -> Option<Self> {
        let file = match fs::OpenOptions::new().read(true).open(SETTINGS_PATH) {
            Ok(file) => file,
            Err(err) => {
                log::critical(format!(
                    "An error occurred while accessing '{}'.
                        The IDE will rely on its default settings. 
                        Reason: {}",
                    SETTINGS_PATH, err
                ));
                return None;
            }
        };

        match serde_json::from_reader(file) {
            Ok(settings) => Some(settings),
            Err(err) => {
                log::critical(format!(
                    "An error occurred while parsing '{}'. 
                        The IDE will rely on its default settings. 
                        Details: {}",
                    SETTINGS_PATH, err
                ));
                None
            }
        }
    }
}
