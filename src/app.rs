// use egui::{KeyboardShortcut, Pos2};
// #[cfg(not(target_arch = "wasm32"))]
// use rfd::FileDialog;

use std::{collections::HashSet, sync::{Arc, Mutex}, thread, time::{Duration}};

#[cfg(target_arch = "wasm32")]
use {
    std::cell::RefCell,
    wasm_bindgen::prelude::*,
    wasm_bindgen::JsCast,
    web_sys::{Blob, Url, BlobPropertyBag},
    js_sys::Uint8Array,
};

use crate::{audio::{interface::AudioBackend, PlatformAudioHandler}, show_file_label_with_click};
use crate::shortcut::{interface::ShortcutListener, PlatformShortcutListener};
use crate::shortcut::keycodes::SerializableKeycode;
use crate::widgets::shortcut::PlatformShortcutRecorder;
use crate::widgets::settings::SettingsWindow;
use crate::sound::Sound;
use crate::utils::*;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct Noisette {
    sounds: Arc<Mutex<Vec<Sound>>>,

    #[serde(skip)]
    audio: Arc<Mutex<PlatformAudioHandler>>,
    #[serde(skip)]
    last_pressed_keys: Option<Vec<SerializableKeycode>>,
    #[serde(skip)]
    shortcut_listener: Arc<Mutex<PlatformShortcutListener>>,

    settings: Arc<Mutex<SettingsWindow>>,

    #[serde(skip)]
    current_playing: Arc<Mutex<HashSet<usize>>>,
    dragging_index: Option<usize>,
    listening_shortcut: Option<usize>
}

impl Default for Noisette {
    fn default() -> Self {
        Self {
            sounds: Arc::new(Mutex::new(Vec::new())),
            settings: Arc::new(Mutex::new(SettingsWindow::default())),
            listening_shortcut: None,
            dragging_index: None,
            audio: Arc::new(Mutex::new(PlatformAudioHandler::new())),
            current_playing: Arc::new(Mutex::new(HashSet::new())),
            shortcut_listener: Arc::new(Mutex::new(PlatformShortcutListener::new())),
            last_pressed_keys: None,
        }
    }
}

impl Noisette {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let instance = if let Some(storage) = cc.storage {
            let instance: Noisette = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();

            if let Ok(settings) = instance.settings.lock() && let Some(device_name) = settings.selected_device_name.clone() {
                if let Ok(mut audio) = instance.audio.lock() {
                    audio.set_device(Some(device_name));
                }
            }

            instance
        } else {
            Noisette::default()
        };

        #[cfg(not(target_arch = "wasm32"))]
        {
            let shortcut_listener = Arc::clone(&instance.shortcut_listener);
            let sounds = Arc::clone(&instance.sounds);
            let audio = Arc::clone(&instance.audio);
            let settings = Arc::clone(&instance.settings);
            let current_playing = Arc::clone(&instance.current_playing);

            thread::spawn(move || {

                loop {
                    if let (Ok(mut listener), Ok(mut sounds)) = (shortcut_listener.lock(), sounds.lock()) {
                        listener.update();

                        for idx in 0..sounds.len() {
                            let sound = &mut sounds[idx];

                            if let Some(shortcut) = &sound.shortcut 
                            && let Ok(audio) = &mut audio.lock() 
                            && let Ok(mut current_playing) = current_playing.lock() {
                                if !listener.is_pressed(&shortcut) {
                                    // Se non e' premuta la shortcut del suono non fare nulla
                                    continue;
                                };

                                if sound.editing {
                                    // Se il suono e' in modalita' modifica non fare nulla
                                    continue;
                                }

                                if audio.is_playing() {
                                    // Se c'e' gia' un suono in riproduzione
                                    if let Ok(settings) = settings.lock() {
                                        if settings.toggle_to_stop && current_playing.contains(&idx) {
                                            // e c'e' l'opzione di premere una seconda volta per stoppare l'audio
                                            // e l'audio attuale e' uguale a quello in riproduzione
                                            audio.stop(sound);
                                            current_playing.remove(&idx);
                                            thread::sleep(Duration::from_millis(1000));
                                            continue;
                                        }

                                        if settings.stop_on_new  && !current_playing.contains(&idx) {
                                            // e c'e' l'opzione di premere un altra shortcut per iniziare un altro audi
                                            // e l'audio attuale e' diverso da quello in riproduzione

                                            current_playing.insert(idx);
                                            audio.play(sound);
                                            thread::sleep(Duration::from_millis(1000));
                                            continue;
                                        }
                                    }

                                    continue;
                                }

                                // Se non c'e' nessun audio in riproduzione
                                // e il suono attuale e' premuto
                                // e il suono attuale non e' in modalita' modifica

                                current_playing.insert(idx);
                                audio.play(sound);
                                thread::sleep(Duration::from_millis(1000));
                                break;
                            }
                        }
                    }

                    thread::sleep(Duration::from_millis(50));
                }
            });
        }

        instance
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
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let mut sounds = self.sounds.lock().unwrap();
        let mut audio = self.audio.lock().unwrap();

        #[cfg(target_arch = "wasm32")]
        {
            LAST_SOUND.with(|slot| {
                if let Some((row, name, data)) = slot.borrow_mut().take() {
                    if let Some(sound) = sounds.get_mut(row) {
                        sound.path = Some(name);
                        sound.data = Some(data.to_vec());
                    }
                }
            });
        }

        if let Ok(mut settings) = self.settings.lock() && let Some(new_device) = &mut settings.new_device {
            println!("Device changed to: {new_device}");
            audio.set_device(Some(new_device.clone()));
            settings.new_device = None;
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                if ui.button("Add").clicked() {
                    let mut default_sound = Sound::default();
                    default_sound.editing = true;
                    sounds.push(default_sound);
                }

                if ui.button("Settings").clicked() && let Ok(mut settings) = self.settings.lock() {
                    settings.open = !settings.open;
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    egui::widgets::global_theme_preference_buttons(ui);
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let column_count = 6;

            ui.columns(column_count, |columns| {
                columns[0].add_sized([0.0, 2.0],egui::Label::new("Name"));
                columns[1].add_sized([0.0, 2.0],egui::Label::new("Shortcut"));
                columns[2].add_sized([0.0, 2.0],egui::Label::new("File"));
                columns[3].add_sized([0.0, 2.0],egui::Label::new("Select / Play"));
                columns[4].add_sized([0.0, 2.0],egui::Label::new("Edit / Save"));
                columns[5].add_sized([0.0, 2.0],egui::Label::new("Remove"));
            });
            ui.separator();

            let mut to_remove = None;

            egui::ScrollArea::vertical()
                .auto_shrink([false; 2]) // evita che si restringa troppo
                .show(ui, |ui| {
                    for idx in 0..sounds.len() {
                        let sound = &mut sounds[idx];
                        ui.columns(column_count, |columns| {
                            if sound.editing {
                                if sound.name.is_none() {
                                    sound.name = Some(String::new());
                                }

                                // Name
                                columns[0].horizontal(|ui| {
                                    ui.text_edit_singleline(sound.name.as_mut().unwrap());
                                });

                                // ShortcutRecorder
                                columns[1].add_sized(
                                    [0.0, 2.0],
                                    PlatformShortcutRecorder::new(
                                        &mut sound.shortcut,
                                        &mut self.listening_shortcut,
                                        &mut self.last_pressed_keys,
                                        idx
                                ));

                                show_file_label_with_click(&mut columns[2], sound);

                                // Select File
                                if columns[3].add_sized([0.0, 2.0],egui::Button::new("Select File").min_size(egui::Vec2::ZERO)).clicked() {
                                    #[cfg(not(target_arch = "wasm32"))]
                                    {
                                        if let Some(path) = rfd::FileDialog::new().add_filter("Audio", &["mp3", "wav"]).pick_file() {
                                            sound.path = Some(path.display().to_string());
                                        }
                                    }
                                    #[cfg(target_arch = "wasm32")]
                                    {
                                        trigger_file_picker(idx);
                                    }
                                }

                                // Save
                                if columns[4].add_sized([0.0, 2.0],egui::Button::new("Save").min_size(egui::Vec2::ZERO)).clicked() {
                                    sound.editing = false;
                                }

                                // Remove
                                if columns[5].add_sized([0.0, 2.0], egui::Button::new("Remove").min_size(egui::Vec2::ZERO)).clicked() {
                                    to_remove = Some(idx);
                                }
                            } else {
                                let label = sound.name.as_deref().filter(|s| !s.is_empty()).unwrap_or("No Name");

                                // Name
                                columns[0].add_sized(
                                    [0.0, 2.0],
                                    egui::Label::new(label).wrap_mode(egui::TextWrapMode::Truncate)
                                );

                                // Shortcut
                                columns[1].add_sized(
                                    [0.0, 2.0],
                                    egui::Label::new(&sound.shortcut
                                        .clone()
                                        .map_or("No Shortcut".to_string(), |s| shortcut_as_string(&s)))
                                        .wrap_mode(egui::TextWrapMode::Truncate)
                                );

                                // File name
                                show_file_label_with_click(&mut columns[2], sound);

                                // Play / Stop button
                                if let Ok(mut current_playing) = self.current_playing.lock() {
                                    if current_playing.contains(&idx) {
                                        if columns[3].add_sized([0.0, 2.0], egui::Button::new("Stop")).clicked() {
                                            audio.stop(sound);
                                            current_playing.remove(&idx);
                                        }
                                    } else {
                                        if columns[3].add_sized([0.0, 2.0], egui::Button::new("Play")).clicked() {
                                            audio.play(sound);
                                            current_playing.insert(idx);
                                        }
                                    }
                                }

                                // Edit
                                if columns[4].add_sized([0.0, 2.0],egui::Button::new("Edit")).clicked() {
                                    sound.editing = true;
                                }

                                // Remove
                                if columns[5].add_sized([0.0, 2.0],egui::Button::new("Remove")).clicked() {
                                    to_remove = Some(idx);
                                }
                            }
                        });
                    }
                });

            if let Some(idx) = to_remove {
                sounds.remove(idx);
            }

            if let Ok(mut settings) = self.settings.lock() && settings.open {
                settings.show(ctx, frame);
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                egui::warn_if_debug_build(ui);

                if cfg!(debug_assertions) && ui.button("Clear Data").clicked() {
                    sounds.clear();
                }
            });
        });
    }
}
