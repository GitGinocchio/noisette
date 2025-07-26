use crate::Sound;
use std::path::{Path};

pub fn show_file_label_with_click(ui: &mut egui::Ui, sound: &Sound) {
    match sound.path.as_deref() {
        Some(path) => {
            let path_buf = Path::new(path);
            let file_name = path_buf
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("No file");
            let mut file_exists = true;

            #[cfg(target_os = "windows")]
            {
                use std::fs::{exists};

                if !exists(path_buf).expect("errore verificando l'esistenza del file") {
                    file_exists = false;
                }
            }

            let label = egui::Label::new(file_name)
                .wrap_mode(egui::TextWrapMode::Truncate)
                .sense(egui::Sense::click());

            let response = if file_exists {
                ui.add_sized([0.0, 2.0],label)
            } else {
                egui::Frame::new()
                    .fill(egui::Color32::from_rgba_unmultiplied(255, 50, 50, 50))
                    .corner_radius(egui::CornerRadius::same(2))
                    .show(ui, |ui| {
                        ui.add_sized([0.0, 2.0],label)
                            .on_hover_text("File not found!");
                    }).response
            };

            if response.hovered() {
                ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
            }

            if response.clicked() && file_exists {
                #[cfg(target_os = "windows")]
                {
                    if let Some(p) = path_buf.to_str() {
                        let _ = std::process::Command::new("explorer")
                            .arg("/select,")
                            .arg(p)
                            .status();
                    }
                }

                #[cfg(target_os = "macos")]
                {
                    let _ = std::process::Command::new("open")
                        .arg("-R")
                        .arg(path_buf)
                        .status();
                }

                #[cfg(target_os = "linux")]
                {
                    if let Some(parent) = path_buf.parent() {
                        let _ = std::process::Command::new("xdg-open")
                            .arg(parent)
                            .status();
                    }
                }

                #[cfg(target_arch = "wasm32")]
                {
                    use js_sys::{Array, Uint8Array};
                    use wasm_bindgen::JsCast;
                    use web_sys::{Blob, BlobPropertyBag, Url, HtmlAnchorElement, Window, Document};


                    if let Some(data) = &sound.data {
                        let array = Uint8Array::from(data.as_slice());

                        let bag = BlobPropertyBag::new();
                        bag.set_type("audio/wav");

                        let blob = Blob::new_with_u8_array_sequence_and_options(
                            &js_sys::Array::of1(&array.into()),
                            &bag,
                        )
                        .unwrap();
                        let url = Url::create_object_url_with_blob(&blob).unwrap();

                        let window = web_sys::window().unwrap();
                        let document = window.document().unwrap();
                        let a = document
                            .create_element("a")
                            .unwrap()
                            .dyn_into::<web_sys::HtmlAnchorElement>()
                            .unwrap();
                        a.set_href(&url);
                        a.set_download(file_name);
                        a.click();

                        Url::revoke_object_url(&url).unwrap();
                    }
                }
            }
        }
        None => {
            ui.add_sized(
                [0.0, 2.0],
                egui::Label::new("No File")
            );
        }
    }
}