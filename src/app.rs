//use egui::{KeyboardShortcut, Pos2};

//#[cfg(not(target_arch = "wasm32"))]
//use rfd::FileDialog;

#[cfg(target_arch = "wasm32")]
use {
    std::cell::RefCell,
    wasm_bindgen::prelude::*,
    wasm_bindgen::JsCast,
    web_sys::{Blob, Url, BlobPropertyBag},
    js_sys::Uint8Array,
};



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

#[cfg(target_arch = "wasm32")]
thread_local! {
    pub static LAST_SOUND: RefCell<Option<(usize, String, Uint8Array)>> = RefCell::new(None);
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn handle_file(row : usize, name: String, data: Uint8Array) {
    println!("{row}");
    LAST_SOUND.with(|slot| {
        *slot.borrow_mut() = Some((row, name, data));
    });
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

        #[cfg(target_arch = "wasm32")]
        {
            LAST_SOUND.with(|slot| {
                if let Some((row, name, data)) = slot.borrow_mut().take() {
                    if let Some(sound) = self.sounds.get_mut(row) {
                        let sound: &mut Sound = sound;
                        sound.path = Some(name);
                        sound.data = Some(data.to_vec());
                    } else {
                        // Index non valido (es. fuori range)
                        eprintln!("Errore: index {} fuori dai limiti di sounds (len = {})", row, self.sounds.len());
                    }
                }
            });
        }


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
            //let height = ui.available_height();

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

                        //let drag_id = egui::Id::new(format!("drag_sound_{}", idx));
                        //let is_dragging = ctx.is_being_dragged(drag_id);

                        //let row_start = ui.cursor().min;

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
                                    
                                        #[cfg(target_arch = "wasm32")]
                                        {
                                            if let Some(data) = &sound.data {
                                                let array = Uint8Array::from(data.as_slice());

                                                let bag = BlobPropertyBag::new();
                                                bag.set_type("audio/wav"); // o audio/mp3, ecc

                                                let blob = Blob::new_with_u8_array_sequence_and_options(&js_sys::Array::of1(&array.into()), &bag).unwrap();
                                                let url = Url::create_object_url_with_blob(&blob).unwrap();

                                                let window = web_sys::window().unwrap();
                                                let document = window.document().unwrap();
                                                let a = document.create_element("a").unwrap().dyn_into::<web_sys::HtmlAnchorElement>().unwrap();
                                                a.set_href(&url);
                                                a.set_download(file_name);
                                                a.click();

                                                Url::revoke_object_url(&url).unwrap();
                                            }
                                        }
                                    };
                                },
                                None => {
                                    ui.add_sized([col_width, 20.0], egui::Label::new("No File"));
                                }
                            };

                            if ui.add_sized([col_width, 20.0], egui::Button::new("Select File")).clicked() {
                                #[cfg(not(target_arch = "wasm32"))]
                                {
                                    if let Some(path) = rfd::FileDialog::new()
                                        .add_filter("Audio files", &["mp3", "wav"])
                                        .pick_file()
                                    {
                                        sound.path = Some(path.display().to_string());
                                    }
                                }
                                #[cfg(target_arch = "wasm32")] 
                                {
                                    trigger_file_picker(idx);
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
                                        
                                        #[cfg(target_arch = "wasm32")]
                                        {
                                            if let Some(data) = &sound.data {
                                                let array = Uint8Array::from(data.as_slice());

                                                let bag = BlobPropertyBag::new();
                                                bag.set_type("audio/wav"); // o audio/mp3, ecc

                                                let blob = Blob::new_with_u8_array_sequence_and_options(&js_sys::Array::of1(&array.into()), &bag).unwrap();
                                                let url = Url::create_object_url_with_blob(&blob).unwrap();

                                                let window = web_sys::window().unwrap();
                                                let document = window.document().unwrap();
                                                let a = document.create_element("a").unwrap().dyn_into::<web_sys::HtmlAnchorElement>().unwrap();
                                                a.set_href(&url);
                                                a.set_download(file_name);
                                                a.click();

                                                Url::revoke_object_url(&url).unwrap();
                                            }
                                        }
                                    };
                                },
                                None => {
                                    ui.add_sized([col_width, 20.0], egui::Label::new("No File"));
                                }
                            };

                            if ui.add_sized([col_width, 20.0], egui::Button::new("Play")).clicked() {

                            }

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
