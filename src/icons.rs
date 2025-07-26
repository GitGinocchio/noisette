use std::collections::HashMap;
use eframe::egui::{self, TextureHandle, TextureOptions};
use egui::TextureFilter;

// Inclusione delle icone
/*
pub static SAVE_ICON: &[u8] = include_bytes!("../assets/save-icon-24.png");
pub static LOAD_ICON: &[u8] = include_bytes!("../assets/load-icon-24.png");
pub static REMOVE_ICON: &[u8] = include_bytes!("../assets/remove-icon-24.png");
pub static PLAY_ICON: &[u8] = include_bytes!("../assets/play-icon-24.png");
pub static STOP_ICON: &[u8] = include_bytes!("../assets/stop-icon-24.png");
pub static EDIT_ICON: &[u8] = include_bytes!("../assets/edit-icon-24.png");
*/

// Azioni con emoji equivalenti
pub const SAVE_EMOJI: &str = "💾";      // Salva
pub const LOAD_EMOJI: &str = "📂";      // Carica
pub const REMOVE_EMOJI: &str = "🗑";     // Rimuovi
pub const PLAY_EMOJI: &str = "▶";       // Riproduci
pub const STOP_EMOJI: &str = "⏹";       // Ferma
pub const EDIT_EMOJI: &str = "✏";       // Modifica

/*
fn load_icon(
    ctx: &egui::Context,
    icon: &[u8],
    icon_name: &str,
    size: Option<[usize; 2]>,
    texture_options: Option<TextureOptions>,
) -> TextureHandle {
    let image = image::load_from_memory(icon).expect("Immagine non valida").to_rgba8();

    let resized_image = if let Some(size) = size {
        image::imageops::resize(&image, size[0] as u32, size[1] as u32, image::imageops::FilterType::Lanczos3)
    } else {
        image
    };

    let size = [resized_image.width() as usize, resized_image.height() as usize];
    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &resized_image);

    let options = texture_options.unwrap_or(TextureOptions {
        magnification: TextureFilter::Linear,
        minification: TextureFilter::Linear,
        ..Default::default()
    });

    ctx.load_texture(icon_name, color_image, options)
}

/// Carica tutte le icone e restituisce un HashMap<nome, TextureHandle>
pub fn load_all_icons(ctx: &egui::Context, size: Option<[usize; 2]>) -> HashMap<String, TextureHandle> {
    let mut icons = HashMap::new();

    let options = TextureOptions {
        magnification: TextureFilter::Linear,
        minification: TextureFilter::Linear,
        ..Default::default()
    };

    icons.insert("save".into(), load_icon(ctx, SAVE_ICON, "save", size, Some(options)));
    icons.insert("load".into(), load_icon(ctx, LOAD_ICON, "load", size, Some(options)));
    icons.insert("remove".into(), load_icon(ctx, REMOVE_ICON, "remove", size, Some(options)));
    icons.insert("play".into(), load_icon(ctx, PLAY_ICON, "play", size, Some(options)));
    icons.insert("stop".into(), load_icon(ctx, STOP_ICON, "stop", size, Some(options)));
    icons.insert("edit".into(), load_icon(ctx, EDIT_ICON, "edit", size, Some(options)));

    icons
}
*/