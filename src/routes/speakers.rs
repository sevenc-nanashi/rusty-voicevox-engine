use crate::{
    models::{SpeakerInfo, StyleInfo},
    resource_manager::GENERATE_PATH,
    result::ErrorJson,
    utils::binary_to_base64,
    vvm_manager::VVM_MANAGER,
};

use axum::{extract::Query, response::IntoResponse, Json};
use futures_util::future::join_all;
use http::StatusCode;
use serde::Deserialize;

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

    match speaker {
        Some(speaker) => Json(SpeakerInfo {
            policy: "本家の規約を参照してください。".to_string(),
            portrait: binary_to_base64(
                tokio::fs::read(format!("{}/{}.png", GENERATE_PATH, speaker.speaker_uuid))
                    .await
                    .unwrap()
                    .as_slice(),
            ),
            style_infos: join_all(
                speaker
                    .styles
                    .iter()
                    .map(|style| async move {
                        let icon = binary_to_base64(
                            tokio::fs::read(format!("{}/{}-{}.png", GENERATE_PATH, speaker.speaker_uuid, style.id()))
                                .await
                                .unwrap()
                                .as_slice(),
                        );
                        StyleInfo {
                            id: style.id() as _,
                            icon,
                            portrait: None,
                            voice_samples: join_all(
                                (0..2)
                                    .map(|i| async move {
                                        binary_to_base64(
                                            tokio::fs::read(format!(
                                                "{}/{}-{}-{}.wav",
                                                GENERATE_PATH,
                                                speaker.speaker_uuid,
                                                style.id(),
                                                i
                                            ))
                                            .await
                                            .unwrap()
                                            .as_slice(),
                                        )
                                    })
                                    .collect::<Vec<_>>(),
                            )
                            .await,
                        }
                    })
                    .collect::<Vec<_>>(),
            )
            .await,
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
