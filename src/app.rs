use egui::{KeyboardShortcut, Pos2};
use rfd::FileDialog;

use crate::widgets::shortcut::ShortcutRecorder;
use crate::sound::Sound;
use crate::utils::*;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct Noisette {
    // Example stuff:
    sounds: Vec<Sound>,

    // #[serde(skip)] // This how you opt-out of serialization of a field

    #[serde(skip)]
    show_add_window: bool,
    
    dragging_index: Option<usize>,
    listening_shortcut: Option<usize>
}

impl Default for Noisette {
    fn default() -> Self {
        Self {
            sounds: Vec::new(),
            show_add_window: false,
            listening_shortcut: None,
            dragging_index: None
        }
    }
}

impl Noisette {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for Noisette {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        println!("Saving!");
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                /*
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }
                */

                if ui.button("Add").clicked() {
                    let mut default_sound = Sound::default();
                    default_sound.editing = true;
                    self.sounds.push(default_sound);
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    egui::widgets::global_theme_preference_buttons(ui);
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            let width = ui.available_width();
            let height = ui.available_height();

            let col_width = (width / 6.0) - 10.0;

            egui::Grid::new("tabella_suoni")
                .striped(true)
                .show(ui, |ui| {
                    ui.add_sized([col_width, 20.0], egui::widgets::Label::new("Name"));
                    ui.add_sized([col_width, 20.0], egui::widgets::Label::new("Shortcut"));
                    ui.add_sized([col_width, 20.0], egui::widgets::Label::new("File"));
                    ui.add_sized([col_width, 20.0],egui::widgets::Label::new("Select File / Play"));
                    ui.add_sized([col_width, 20.0],egui::widgets::Label::new("Edit / Save"));
                    ui.add_sized([col_width, 20.0],egui::widgets::Label::new("Remove"));
                    ui.end_row();

                    let mut to_remove = None;

                    // Per il drag and drop:
                    // https://github.com/emilk/egui/blob/main/crates/egui_demo_lib/src/demo/drag_and_drop.rs

                    for idx in 0..self.sounds.len() {
                        let sound = &mut self.sounds[idx];

                        let drag_id = egui::Id::new(format!("drag_sound_{}", idx));
                        let is_dragging = ctx.is_being_dragged(drag_id);

                        let row_start = ui.cursor().min;

                        if sound.editing {
                            if sound.name.is_none() {
                                sound.name = Some(String::new());
                            }

                            ui.add_sized(
                                [col_width, 20.0],
                                egui::TextEdit::singleline(sound.name.as_mut().unwrap())
                                    .hint_text("Write a name here"),
                            );

                            ui.add_sized(
                                [col_width, 20.0],
                                ShortcutRecorder::new(
                                &mut sound.shortcut,
                                &mut self.listening_shortcut,
                                idx,
                            ));

                            match sound.path.as_deref() {
                                Some(path) => {
                                    let path = std::path::Path::new(path);

                                    let file_name = match path.file_name().and_then(|n| n.to_str()) {
                                        Some(file_name) => file_name,
                                        None => "No file"
                                    };

                                    if ui.add_sized([col_width, 20.0], egui::Button::new(file_name)).clicked() {
                                        #[cfg(target_os = "windows")]
                                        {
                                            if let Some(path_str) = path.to_str() {
                                                let _ = std::process::Command::new("explorer")
                                                    .arg("/select,")
                                                    .arg(path_str)
                                                    .spawn();
                                            }
                                        }

                                        #[cfg(target_os = "macos")]
                                        {
                                            let _ = std::process::Command::new("open")
                                                .arg("-R")
                                                .arg(path)
                                                .spawn();
                                        }

                                        #[cfg(target_os = "linux")]
                                        {
                                            if let Some(parent) = path.parent() {
                                                let _ = std::process::Command::new("xdg-open")
                                                    .arg(parent)
                                                    .spawn();
                                            }
                                        } 
                                    };
                                },
                                None => {
                                    ui.add_sized([col_width, 20.0], egui::Label::new("No File"));
                                }
                            };

                            if ui.add_sized([col_width, 20.0], egui::Button::new("Select File")).clicked() {
                                if let Some(path) = rfd::FileDialog::new()
                                    .add_filter("Audio files", &["mp3", "wav"])
                                    .pick_file()
                                {
                                    sound.path = Some(path.display().to_string());
                                }
                            }

                            if ui.add_sized([col_width, 20.0], egui::Button::new("Save")).clicked() {
                                sound.editing = false;
                            }

                            if ui.add_sized([col_width, 20.0], egui::Button::new("Remove")).clicked() {
                                to_remove = Some(idx);
                            }
                        } 
                        else {
                            let sound_name = match sound.name.as_deref() {
                                Some(string) => {
                                    if string.chars().count() > 0 { string }
                                    else { "No Name" }
                                }
                                None => "No Name"
                            };

                            ui.add_sized([col_width, 20.0], egui::Label::new(sound_name));

                            ui.add_sized([col_width, 20.0], egui::Label::new(&sound.shortcut.map_or_else(
                                || "No Shortcut".to_string(),
                                |s| shortcut_as_string(&s),
                            )));

                            match sound.path.as_deref() {
                                Some(path) => {
                                    let path = std::path::Path::new(path);

                                    let file_name = match path.file_name().and_then(|n| n.to_str()) {
                                        Some(file_name) => file_name,
                                        None => "No file"
                                    };

                                    if ui.add_sized([col_width, 20.0], egui::Button::new(file_name)).clicked() {
                                        #[cfg(target_os = "windows")]
                                        {
                                            if let Some(path_str) = path.to_str() {
                                                let _ = std::process::Command::new("explorer")
                                                    .arg("/select,")
                                                    .arg(path_str)
                                                    .spawn();
                                            }
                                        }

                                        #[cfg(target_os = "macos")]
                                        {
                                            let _ = std::process::Command::new("open")
                                                .arg("-R")
                                                .arg(path)
                                                .spawn();
                                        }

                                        #[cfg(target_os = "linux")]
                                        {
                                            if let Some(parent) = path.parent() {
                                                let _ = std::process::Command::new("xdg-open")
                                                    .arg(parent)
                                                    .spawn();
                                            }
                                        } 
                                    };
                                },
                                None => {
                                    ui.add_sized([col_width, 20.0], egui::Label::new("No File"));
                                }
                            };

                            ui.add_sized([col_width, 20.0], egui::Button::new("Play"));

                            if ui.add_sized([col_width, 20.0], egui::Button::new("Edit")).clicked() {
                                sound.editing = true;
                            }

                            if ui.add_sized([col_width, 20.0], egui::Button::new("Remove")).clicked() {
                                to_remove = Some(idx);
                            }
                        }

                        ui.end_row();
                    }

                if let Some(idx) = to_remove {
                    self.sounds.remove(idx);
                }
            });

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                egui::warn_if_debug_build(ui);
                if ui.button("Clear Data").clicked() {
                    self.sounds.clear();
                }
            });
        });
    }
}
