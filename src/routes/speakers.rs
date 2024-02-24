use crate::{
    models::{SpeakerInfo, StyleInfo},
    resource_manager::RESOURCE_MANAGER,
    result::ErrorJson,
    utils::image_to_base64,
    vvm_manager::VVM_MANAGER,
};

use axum::{extract::Query, response::IntoResponse, Json};
use http::StatusCode;
use serde::Deserialize;
use std::sync::Arc;

pub async fn speakers_get() -> impl IntoResponse {
    let vvm_manager = VVM_MANAGER.get().unwrap().lock().await;
    let speakers = vvm_manager.speakers();

    Json(speakers).into_response()
}

#[derive(Deserialize)]
pub struct SpeakerInfoGetQuery {
    speaker_uuid: String,
}

pub async fn speaker_info_get(Query(query): Query<SpeakerInfoGetQuery>) -> impl IntoResponse {
    let vvm_manger = VVM_MANAGER.get().unwrap().lock().await;
    let speaker = vvm_manger.speaker(&query.speaker_uuid);
    let resource_manager = Arc::clone(RESOURCE_MANAGER.get().unwrap());
    let resource_manager = resource_manager.lock().await;

    match speaker {
        Some(speaker) => Json(SpeakerInfo {
            policy: "本家の規約を参照してください。".to_string(),
            portrait: image_to_base64(resource_manager.portrait_image(&speaker.speaker_uuid).unwrap()),
            style_infos: speaker
                .styles
                .iter()
                .map(|style| {
                    let icon = image_to_base64(resource_manager.style_icon(style.id()).unwrap());
                    StyleInfo {
                        id: style.id() as _,
                        icon,
                        portrait: None,
                        voice_samples: vec!["".to_string(), "".to_string(), "".to_string()],
                    }
                })
                .collect(),
        })
        .into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(ErrorJson {
                error: "Speaker not found".to_string(),
            }),
        )
            .into_response(),
    }
}
