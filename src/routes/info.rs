use crate::{
    models::{EngineManifest, SupportedFeatures},
    result::Result,
};

use axum::Json;
use serde::{Deserialize, Serialize};
use voicevox_core_rs::SupportedDevices;

#[derive(Serialize, Deserialize)]
pub struct Index {
    version: String,
}

pub async fn index_get() -> Json<Index> {
    Json(Index {
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}
pub async fn engine_manifest_get() -> Json<EngineManifest> {
    let manifest = EngineManifest {
        manifest_version: "0.13.1".to_string(),
        name: "Rusty Voicevox Engine".to_string(),
        brand_name: "Rusty Voicevox".to_string(),
        uuid: "6d7c1608-538a-4f55-9055-7452d8e58025".to_string(),
        url: "https://github.com/sevenc-nanashi/rusty-voicevox-engine".to_string(),
        icon: "https://raw.githubusercontent.com/sevenc-nanashi/rusty-voicevox-engine/main/assets/icon.png".to_string(),
        default_sampling_rate: 24000,
        terms_of_service: include_str!("../../assets/terms_of_service.md").to_string(),
        dependency_licenses: vec![],
        update_infos: vec![],
        supported_features: SupportedFeatures {
            adjust_mora_pitch: true,
            adjust_phoneme_length: true,
            adjust_speed_scale: true,
            adjust_pitch_scale: true,
            adjust_intonation_scale: true,
            adjust_volume_scale: true,
            interrogative_upspeak: true,
            synthesis_morphing: false,
            manage_library: false,
        },
    };

    Json(manifest)
}

pub async fn version_get() -> Json<String> {
    Json(env!("CARGO_PKG_VERSION").to_string())
}

pub async fn supported_devices_get() -> Result<Json<SupportedDevices>> {
    Ok(Json(SupportedDevices::get().map_err(anyhow::Error::from)?))
}
