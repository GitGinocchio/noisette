use egui::*;

use crate::audio::{
    get_output_devices,
    get_default_output_device
};

#[derive(serde::Deserialize, serde::Serialize)]
#[derive(PartialEq, Clone, Copy)]
pub enum SettingsTab {
    General,
    Audio,
    Video,
}

impl std::fmt::Display for SettingsTab {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            SettingsTab::General => "General",
            SettingsTab::Audio => "Audio",
            SettingsTab::Video => "Video",
        };
        write!(f, "{}", s)
    }
}


#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct SettingsWindow {
    pub open: bool,
    selected_tab: SettingsTab,
    pub output_devices: Vec<String>,
    pub selected_device_name: Option<String>,
    pub toggle_to_stop: bool,
    pub stop_on_new: bool,
    
    #[serde(skip)]
    pub new_device: Option<String>
}

impl Default for SettingsWindow {
    fn default() -> Self {
        Self {
            open: false,
            selected_tab: SettingsTab::General,
            output_devices: get_output_devices(),
            selected_device_name: get_default_output_device(),
            toggle_to_stop: true,
            stop_on_new: true,
            new_device: None
        }
    }
}

impl SettingsWindow {
    pub fn show(&mut self, ctx: &Context, _frame: &eframe::Frame) {
        if !self.open {
            return;
        }

        Window::new("Settings")
            .open(&mut self.open)
            .show(ctx, |ui| {
                // Tabs
                ui.horizontal(|ui| {
                    for tab in [SettingsTab::General, SettingsTab::Audio, SettingsTab::Video] {
                        if ui
                            .selectable_label(self.selected_tab == tab, tab.to_string())
                            .clicked()
                        {
                            self.selected_tab = tab;
                        }
                    }
                });

                ui.separator();

                match self.selected_tab {
                    SettingsTab::General => {
                        ui.checkbox(&mut self.toggle_to_stop, "Toggle to stop")
                            .on_hover_text("Clicking again on a playing sound will stop it.");

                        ui.checkbox(&mut self.stop_on_new, "Interrupt on new")
                            .on_hover_text("Stops the currently playing sound when a new one is triggered.");

                        ui.label(
                            RichText::new("ℹ Hover over a setting to see its description")
                                .small()
                                .color(ui.visuals().weak_text_color()),
                        );
                    }
                    SettingsTab::Audio => {
                        ui.label("Output Audio Device:");
                        egui::ComboBox::from_id_salt("output_audio_device")
                            .selected_text(
                                self.selected_device_name
                                    .as_deref()
                                    .unwrap_or("<no device selected>"),
                            )
                            .show_ui(ui, |ui| {
                                for device_name in &self.output_devices {
                                    if ui.selectable_value(
                                        &mut self.selected_device_name, 
                                        Some(device_name.clone()), 
                                        device_name
                                    ).clicked() {
                                        self.new_device = Some(device_name.clone());
                                    }
                                }
                            });
                    },
                    SettingsTab::Video => { 
                        ui.label(
                            RichText::new("⚠ Work in progress")
                            .color(ui.visuals().warn_fg_color),
                        );
                    }
                }
            });
    }
}
