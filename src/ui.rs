///! CodeEditor and its implementations, with some helper functions.
///! The CodeEditor is reponsible for rendering and handling events and keyboard inputs.
use eframe::egui;
use std::ffi;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process;

use super::highligher::{Highligher, Token, TokenType};
use super::settings::{CodeColor, Settings};

pub struct CodeEditor {
    /// Code contents
    contents: String,

    /// No path is set when the editor opens
    path: Option<PathBuf>,

    /// Console contents
    console: String,

    /// Has the file been saved?
    saved: bool,

    /// User settings
    settings: Settings,
}
impl CodeEditor {
    pub fn new() -> Option<Self> {
        let Some(settings) = Settings::get() else {
            return None;  // Could not load settings
        };
        Some(Self {
            contents: String::new(),
            path: None,
            console: String::new(),
            saved: false,
            settings,
        })
    }
}

impl eframe::App for CodeEditor {
    /// Handle the close event, i.e. when the user clicks on the 'x' in the top
    /// right corner.
    /// If the feature of saving on close is on and the source is not empty, save the contents.
    fn on_close_event(&mut self) -> bool {
        if self.settings.save_on_close && !self.contents.is_empty() {
            self.save_file();
        }
        true // A return value of 'true' means we accept the event
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.handle_ctrl_s(ui.input().events.iter());
            self.handle_ctrl_r(ui.input().events.iter());

            self.draw_top_section(ui);

            ui.separator();

            // Remove highlight of widget when hovered
            ui.visuals_mut().widgets.hovered = ui.visuals_mut().widgets.inactive;

            self.draw_code_editor(ui);

            ui.separator();

            self.draw_console(ui);
        });
    }
}

impl CodeEditor {
    fn draw_top_section(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                // Title label
                ui.label(
                    egui::RichText::new(self.set_title())
                        .size(17.0)
                        .monospace()
                        .strong()
                        .color(egui::Color32::WHITE),
                );
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                // Run button
                if ui
                    .button(
                        egui::RichText::new("Run")
                            .size(15.0)
                            .monospace()
                            .color(egui::Color32::WHITE),
                    )
                    .clicked()
                {
                    self.run_file()
                }

                // Save button
                if self.settings.save_btn {
                    if ui
                        .button(
                            egui::RichText::new("Save")
                                .size(15.0)
                                .monospace()
                                .color(egui::Color32::WHITE),
                        )
                        .clicked()
                    {
                        self.save_file()
                    }
                }

                // Open button
                if ui
                    .button(
                        egui::RichText::new("Open")
                            .size(15.0)
                            .monospace()
                            .color(egui::Color32::WHITE),
                    )
                    .clicked()
                {
                    self.open_file()
                }
            });
        });
    }

    /// Leave 15% space for console
    fn draw_code_editor(&mut self, ui: &mut egui::Ui) {
        egui::Resize::default()
            .fixed_size((ui.available_width(), ui.available_height() * 0.85))
            .show(ui, |ui| {
                egui::ScrollArea::both()
                    .id_source("vscroll1")
                    .show(ui, |ui| {
                        // Remove highlight of widget when ckicked (0.0) but leave the text cursor as white
                        ui.visuals_mut().selection.stroke =
                            egui::Stroke::new(0.0, egui::Color32::WHITE);
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                            // Add code lines
                            ui.add_sized(
                                (ui.available_width() * 0.03, ui.available_height()),
                                egui::Label::new(
                                    egui::RichText::new(self.lines())
                                        .color(egui::Color32::WHITE)
                                        .font(egui::FontId::new(
                                            self.settings.code_font_size,
                                            egui::FontFamily::Monospace,
                                        )),
                                ),
                            );
                            let mut layouter =
                                &mut |ui: &egui::Ui, string: &str, _wrap_width: f32| {
                                    let layout_job = highlight_text(
                                        string,
                                        self.settings.code_color,
                                        self.settings.code_font_size,
                                    );
                                    ui.fonts().layout_job(layout_job)
                                };

                            // Add code editor
                            let response = ui.add_sized(
                                (ui.available_width(), ui.available_height()),
                                egui::widgets::TextEdit::multiline(&mut self.contents)
                                    .code_editor()
                                    .layouter(&mut layouter)
                                    .font(egui::TextStyle::Monospace),
                            );
                            if response.changed() {
                                // The source has been modified
                                self.saved = false;
                            }
                        });
                    })
            });
    }

    fn draw_console(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::both()
            .id_source("vscroll2")
            .show(ui, |ui| {
                // Remove white border from console
                ui.visuals_mut().widgets.noninteractive.bg_stroke = egui::Stroke::NONE;
                ui.add_sized(
                    ui.available_size(),
                    egui::TextEdit::multiline(&mut self.console)
                        .code_editor()
                        .font(egui::FontId::new(
                            self.settings.console_font_size,
                            egui::FontFamily::Monospace,
                        ))
                        .interactive(false),
                );
            });
    }
}

impl CodeEditor {
    /// Return the numbers of the lines on the top left of the editor
    fn lines(&self) -> String {
        // + 1 because we add one newline at least
        let row_count = self.contents.chars().filter(|ch| ch == &'\n').count() + 1;
        let mut lines = (1..=row_count).fold(String::new(), |acc, n| format!("{}\n{}", acc, n));
        lines.remove(0); // Remove the first newline caused by `fold`

        // If we don't do this shitty thing, the label gets pushed in the middle.
        // Therefore, we add as many newlines as we need to fill the ui (empirical count)
        // It looks weird but at least it works :(
        if row_count < 35 {
            let delta = 35 - row_count;
            for _ in 0..delta {
                lines.push('\n');
            }
        }
        lines
    }

    /// If there is a file loaded, we want to show whether the path was saved or not.
    /// Add a '+' if the file has been saved or '-' if not.
    fn set_title(&self) -> String {
        match self.path {
            Some(ref path) if self.saved => format!("+ {}", path_name_as_string(path)),
            Some(ref path) if !self.saved => format!("- {}", path_name_as_string(path)),
            _ => "No file loaded".into(),
        }
    }

    /// A Ctrl+S event is accepted if:
    ///     - Ctrl is pressed
    ///     - S is pressed
    ///     - The current file is not saved
    fn handle_ctrl_s(&mut self, events: std::slice::Iter<'_, egui::Event>) {
        for event in events {
            if matches!(event, egui::Event::Key { key, pressed, modifiers }
            if *pressed
                && matches!(key, egui::Key::S)
                && modifiers.ctrl
                && !self.saved
            ) {
                self.save_file();
            }
        }
    }
    /// A Ctrl+R event is accepted if:
    ///     - Ctrl is pressed
    ///     - R is pressed
    ///     - The current file is not saved
    fn handle_ctrl_r(&mut self, events: std::slice::Iter<'_, egui::Event>) {
        for event in events {
            if matches!(event, egui::Event::Key { key, pressed, modifiers }
            if *pressed
                && matches!(key, egui::Key::R)
                && modifiers.ctrl
                && !self.saved
            ) {
                self.run_file();
            }
        }
    }

    /// Handler for saving the current contents
    fn save_file(&mut self) {
        let path = match self.path {
            Some(ref path) => path.clone(),
            None => {
                // The following only gets the path, does not actually create the file
                let path = rfd::FileDialog::new()
                    .add_filter("betty file", &["betty"])
                    .add_filter("Other files", &["*"])
                    .set_title("Create file")
                    .save_file();
                match path {
                    // Otherwise we cannot live long enough
                    Some(path) => {
                        self.path = Some(path.clone());
                        path
                    }
                    // The user exited the file dialog
                    None => return,
                }
            }
        };

        self.save_file_contents(path);
    }

    /// Run the current file
    fn run_file(&mut self) {
        if self.settings.save_and_run {
            self.save_file();
        }

        let Some(ref path) = self.path else {
            return;
        };

        match run_betty(path, &self.settings.betty_exe_path) {
            Ok(output) => {
                // Combine stdout and stderr as one output
                let contents = format!(
                    "{}{}",
                    String::from_utf8_lossy(&output.stdout).into_owned(),
                    String::from_utf8_lossy(&output.stderr).into_owned()
                );
                self.console = contents
            }
            Err(err) => msgbox(
                "Program execution error",
                err.to_string().as_str(),
                rfd::MessageLevel::Error,
            ),
        }
    }

    /// Open file handler
    fn open_file(&mut self) {
        let Some(path) = rfd::FileDialog::new().pick_file() else {
            // The user exited the file dialog
            return;
        };

        match fs::read_to_string(&path) {
            Ok(contents) => {
                // As the file has just been loaded, it is unmodified
                // and therefore it is considered saved
                self.saved = true;
                self.path = Some(path);
                self.contents = contents;
            }
            Err(err) => msgbox(
                &format!("Error in opening file '{}'", path_name_as_string(&path)),
                err.to_string().as_str(),
                rfd::MessageLevel::Error,
            ),
        }
    }

    /// Save self.contents into 'path
    fn save_file_contents(&mut self, path: PathBuf) {
        match fs::OpenOptions::new()
            .write(true)
            .create(true)
            // .truncate(true)  This is not needed imho
            .open(&path)
        {
            Ok(mut file) => {
                if let Err(err) = file.write_all(self.contents.as_bytes()) {
                    msgbox(
                        &format!("Error in writing to file '{}'", path_name_as_string(&path)),
                        err.to_string().as_str(),
                        rfd::MessageLevel::Error,
                    );
                } else {
                    self.saved = true;
                }
            }
            Err(err) => msgbox(
                "Error in opening file",
                err.to_string().as_str(),
                rfd::MessageLevel::Error,
            ),
        }
    }
}

/// Highlighter of the source code
#[inline]
fn highlight_text(text: &str, code_color: CodeColor, font_size: f32) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    if text.is_empty() {
        return job;
    }

    // Get the tokens from the syntax highligher
    let highlighter = Highligher::new(text.chars().collect());
    let tokens = highlighter.make_tokens();

    // For each token, convert the type into a color
    for token in tokens {
        let Token(typ, literal) = token;
        let color = match typ {
            TokenType::Num => egui::Color32::from_code_color(code_color.number),
            TokenType::Ident => egui::Color32::from_code_color(code_color.ident),
            TokenType::Str => egui::Color32::from_code_color(code_color.string),
            TokenType::Sym => egui::Color32::from_code_color(code_color.symbol),
            TokenType::Kw => egui::Color32::from_code_color(code_color.keyword),
            TokenType::BuiltinFun => egui::Color32::from_code_color(code_color.builtin_fn),
            TokenType::Fun => {
                let [r, g, b, a] = code_color.fun;
                egui::Color32::from_rgba_premultiplied(r, g, b, a)
            }
            TokenType::Comment => egui::Color32::from_code_color(code_color.comment),
            TokenType::Error => egui::Color32::from_code_color(code_color.error),
            TokenType::Other => egui::Color32::from_code_color(code_color.other),
        };

        // Push the color into the buffer
        job.append(
            &literal,
            0.0,
            egui::text::TextFormat {
                color,
                font_id: egui::FontId::new(font_size, egui::FontFamily::Monospace),
                ..Default::default()
            },
        );
    }

    job
}

#[inline]
fn run_betty(path: &Path, betty_exe_path: &str) -> io::Result<process::Output> {
    process::Command::new("cmd")
        .arg("/C")
        .arg(betty_exe_path)
        .arg(ffi::OsString::from(path))
        .output()
}

/// Spawn a MessageBox with the given title, description and level
fn msgbox(title: &str, descr: &str, level: rfd::MessageLevel) {
    rfd::MessageDialog::new()
        .set_title(title)
        .set_description(descr)
        .set_level(level)
        .set_buttons(rfd::MessageButtons::Ok)
        .show();
}

/// Return the name of a [`Path`] as [`String`]
fn path_name_as_string(path: &Path) -> String {
    path.file_name()
        .map(ffi::OsStr::to_string_lossy)
        .unwrap_or_default()
        .into()
}

/// Fron [u8; 3] to [`egui::Color32`]
trait FromCodeColor {
    fn from_code_color(rgb: [u8; 3]) -> Self;
}

impl FromCodeColor for egui::Color32 {
    #[inline]
    fn from_code_color(rgb: [u8; 3]) -> Self {
        Self::from_rgb(rgb[0], rgb[1], rgb[2])
    }
}
