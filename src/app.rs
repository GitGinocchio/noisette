// use egui::{KeyboardShortcut, Pos2};
// #[cfg(not(target_arch = "wasm32"))]
// use rfd::FileDialog;

use std::{sync::{Arc, Mutex}, thread, time::Duration};

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

use crate::icons::*;

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
    dragging_index: Option<usize>,
    listening_shortcut: Option<usize>,
}

impl Default for Noisette {
    fn default() -> Self {
        Self {
            sounds: Arc::new(Mutex::new(Vec::new())),
            settings: Arc::new(Mutex::new(SettingsWindow::default())),
            listening_shortcut: None,
            dragging_index: None,
            audio: Arc::new(Mutex::new(PlatformAudioHandler::new())),
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

            thread::spawn(move || {
                loop {
                    if let (Ok(mut listener), Ok(mut sounds)) = (shortcut_listener.lock(), sounds.lock()) {
                        listener.update();

                        for idx in 0..sounds.len() {
                            let sound = &mut sounds[idx];

                            if let Some(shortcut) = &sound.shortcut 
                            && let Ok(audio) = &mut audio.lock()  {
                                if !listener.is_pressed(&shortcut) {
                                    // Se non e' premuta la shortcut del suono non fare nulla
                                    continue;
                                };

                                if sound.editing {
                                    // Se il suono e' in modalita' modifica non fare nulla
                                    continue;
                                }

                                if audio.is_playing(None) {
                                    // Se c'e' gia' un suono in riproduzione
                                    if let Ok(settings) = settings.lock() {
                                        if settings.toggle_to_stop && audio.is_playing(Some(sound.clone())) {
                                            // e c'e' l'opzione di premere una seconda volta per stoppare l'audio
                                            // e l'audio attuale e' uguale a quello in riproduzione
                                            audio.stop(sound);
                                            thread::sleep(Duration::from_millis(1000));
                                            continue;
                                        }

                                        if settings.stop_on_new  && !audio.is_playing(Some(sound.clone())) {
                                            // e c'e' l'opzione di premere un altra shortcut per iniziare un altro audi
                                            // e l'audio attuale e' diverso da quello in riproduzione
                                            audio.stop_all();
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
                columns[3].add_sized([2.0, 2.0],egui::Label::new("Select / Play"));
                columns[4].add_sized([2.0, 2.0],egui::Label::new("Edit / Save"));
                columns[5].add_sized([2.0, 2.0],egui::Label::new("Remove"));
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
                                
                                /*
                                let select_btn = if let Some(icon) = self.icons.get("load") {
                                    egui::Button::image(icon)
                                }
                                else {
                                    egui::Button::new("Select File")
                                };
                                */
                                let select_btn = egui::Button::new(LOAD_EMOJI);

                                // Select File
                                if columns[3].add_sized([0.0, 2.0], select_btn.min_size(egui::Vec2::ZERO)).clicked() {
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

                                /*
                                let save_btn = if let Some(icon) = self.icons.get("save") {
                                    egui::Button::image(icon)
                                }
                                else {
                                    egui::Button::new("Select File")
                                };
                                */

                                let save_btn = egui::Button::new(SAVE_EMOJI);

                                // Save
                                if columns[4].add_sized([0.0, 2.0],save_btn.min_size(egui::Vec2::ZERO)).clicked() {
                                    sound.editing = false;
                                }

                                /*
                                let remove_btn = if let Some(icon) = self.icons.get("remove") {
                                    egui::Button::image(icon)
                                }
                                else {
                                    egui::Button::new("Remove")
                                };
                                */

                                let remove_btn = egui::Button::new(REMOVE_EMOJI);

                                // Remove
                                if columns[5].add_sized([0.0, 2.0], remove_btn.min_size(egui::Vec2::ZERO)).clicked() {
                                    to_remove = Some(idx);
                                }
                            } 
                            else {
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
                                if audio.is_playing(Some(sound.clone())) {
                                    /*
                                    let stop_btn = if let Some(icon) = self.icons.get("stop") {
                                        egui::Button::image(icon)
                                    }
                                    else {
                                        egui::Button::new("Stop")
                                    };
                                    */
                                    let stop_btn = egui::Button::new(STOP_EMOJI);

                                    if columns[3].add_sized([0.0, 2.0], stop_btn).clicked() {
                                        audio.stop(sound);
                                    }
                                } else {
                                    /*
                                    let play_btn = if let Some(icon) = self.icons.get("play") {
                                        egui::Button::image(icon)
                                    }
                                    else {
                                        egui::Button::new("Play")
                                    };
                                    */

                                    let play_btn = egui::Button::new(PLAY_EMOJI);

                                    if columns[3].add_sized([0.0, 2.0], play_btn).clicked() {
                                        audio.stop_all();
                                        audio.play(sound);
                                    }
                                }

                                /*
                                let edit_btn = if let Some(icon) = self.icons.get("edit") {
                                    egui::Button::image(icon)
                                }
                                else {
                                    egui::Button::new("Edit")
                                };
                                */

                                let edit_btn = egui::Button::new(EDIT_EMOJI);

                                // Edit
                                if columns[4].add_sized([2.0, 2.0],edit_btn).clicked() {
                                    sound.editing = true;
                                }

                                /*
                                let remove_btn = if let Some(icon) = self.icons.get("remove") {
                                    egui::Button::image(icon)
                                }
                                else {
                                    egui::Button::new("Remove")
                                };
                                */

                                let remove_btn = egui::Button::new(REMOVE_EMOJI);

                                // Remove
                                if columns[5].add_sized([2.0, 2.0], remove_btn).clicked() {
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
