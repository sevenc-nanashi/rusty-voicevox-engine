use once_cell::sync::Lazy;
use serde::Deserialize;
use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
};
use tokio::sync::Mutex;
use tracing::info;

use crate::vvm_manager::VVM_MANAGER;

pub struct ResourceManager {
    portrait_images: HashMap<String, image::RgbaImage>,
    style_icons: HashMap<u32, image::RgbaImage>,
}

pub static RESOURCE_MANAGER: OnceLock<Arc<Mutex<ResourceManager>>> = OnceLock::new();
static BORDER: i32 = 5;

static FONT_PATH: &str = if cfg!(feature = "release") {
    "./mplus-2p-semibold.ttf"
} else {
    concat!(env!("CARGO_MANIFEST_DIR"), "/vendored/mplus-2p-semibold.ttf")
};

#[derive(Debug, Deserialize)]
struct SpeakerColor {
    color: (u8, u8, u8),
    light_color: (u8, u8, u8),
}
static COLORS: Lazy<HashMap<String, SpeakerColor>> = Lazy::new(|| {
    let colors: HashMap<String, SpeakerColor> = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/vendored/speaker_colors.json"
    )))
    .unwrap();
    colors
});

impl ResourceManager {
    pub async fn new() -> Self {
        info!("Loading font... {}", FONT_PATH);
        let font = std::fs::read(FONT_PATH).unwrap();
        let font = rusttype::Font::try_from_vec(font).unwrap();
        let speakers = { VVM_MANAGER.get().unwrap().lock().await.speakers().clone() };

        let mut portrait_images = HashMap::new();
        let mut style_icons = HashMap::new();

        for speaker in &speakers {
            info!("Creating image for: {}", speaker.name);
            let mut portrait = image::RgbaImage::new(300, 500);
            let color_info = COLORS.get(&speaker.name).unwrap();
            let color = image::Rgba([color_info.color.0, color_info.color.1, color_info.color.2, 255]);
            let bg_color = image::Rgba([
                color_info.light_color.0,
                color_info.light_color.1,
                color_info.light_color.2,
                128,
            ]);

            imageproc::drawing::draw_filled_rect_mut(
                &mut portrait,
                imageproc::rect::Rect::at(0, 0).of_size(300, 500),
                bg_color,
            );

            imageproc::drawing::draw_filled_rect_mut(
                &mut portrait,
                imageproc::rect::Rect::at(0, 0).of_size(300, BORDER as _),
                color,
            );
            imageproc::drawing::draw_filled_rect_mut(
                &mut portrait,
                imageproc::rect::Rect::at(0, BORDER).of_size(BORDER as _, (500 - BORDER) as _),
                color,
            );
            imageproc::drawing::draw_filled_rect_mut(
                &mut portrait,
                imageproc::rect::Rect::at(0, 500 - BORDER).of_size(300, BORDER as _),
                color,
            );
            imageproc::drawing::draw_filled_rect_mut(
                &mut portrait,
                imageproc::rect::Rect::at(300 - BORDER, BORDER).of_size(BORDER as _, (500 - BORDER) as _),
                color,
            );

            let scale = rusttype::Scale::uniform(50.0);
            let name_size = imageproc::drawing::text_size(scale, &font, &speaker.name);
            let name_x = 150 - (name_size.0 / 2);
            let name_y = 250 - name_size.1 / 2;
            imageproc::drawing::draw_text_mut(&mut portrait, color, name_x, name_y, scale, &font, &speaker.name);

            portrait_images.insert(speaker.speaker_uuid.clone(), portrait);

            for style in &speaker.styles {
                info!("  Creating icon for: {}", style.name());
                let mut icon = image::RgbaImage::new(256, 256);
                let color = image::Rgba([color_info.color.0, color_info.color.1, color_info.color.2, 255]);
                imageproc::drawing::draw_filled_rect_mut(
                    &mut icon,
                    imageproc::rect::Rect::at(0, 0).of_size(256, 256),
                    bg_color,
                );
                let scale = rusttype::Scale::uniform(200.0);
                let size = imageproc::drawing::text_size(scale, &font, &style.id().to_string());
                let x = 128 - size.0 / 2;
                let y = 128 - size.1 / 2 - 32;
                imageproc::drawing::draw_text_mut(&mut icon, color, x, y, scale, &font, &style.id().to_string());

                style_icons.insert(style.id(), icon);
            }
        }

        info!(
            "Created images: {} portrait images, {} style icons",
            portrait_images.len(),
            style_icons.len()
        );

        ResourceManager {
            portrait_images,
            style_icons,
        }
    }

    pub fn portrait_image(&self, speaker_uuid: &str) -> Option<&image::RgbaImage> {
        self.portrait_images.get(speaker_uuid)
    }
    pub fn style_icon(&self, style_id: u32) -> Option<&image::RgbaImage> {
        self.style_icons.get(&style_id)
    }
}
