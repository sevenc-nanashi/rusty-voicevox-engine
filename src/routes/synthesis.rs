use axum::{extract::Query, Json};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use tokio::sync::Mutex;
use voicevox_core_rs::{AccelerationMode, InitializeOptions, OpenJtalkRc, SynthesisOptions, Synthesizer};

use crate::{
    models::{AccentPhrase, AudioQuery},
    result::Result,
    vvm_manager::VVM_MANAGER,
};

pub struct SendSyncOpenJtalk(pub OpenJtalkRc);
unsafe impl Send for SendSyncOpenJtalk {}
unsafe impl Sync for SendSyncOpenJtalk {}

pub struct SendSyncSynthesizer(pub Synthesizer);
unsafe impl Send for SendSyncSynthesizer {}
unsafe impl Sync for SendSyncSynthesizer {}

static DICT_DIR: &str = if cfg!(feature = "release") {
    "./open_jtalk_dic_utf_8-1.11"
} else {
    concat!(env!("CARGO_MANIFEST_DIR"), "/vendored/open_jtalk_dic_utf_8-1.11")
};
pub static OPEN_JTALK: Lazy<Mutex<SendSyncOpenJtalk>> = Lazy::new(|| {
    let open_jtalk = OpenJtalkRc::new(DICT_DIR).unwrap();
    Mutex::new(SendSyncOpenJtalk(open_jtalk))
});
pub async fn init_synthesizer(use_gpu: bool, cpu_num_threads: usize) {
    let synthesizer = Synthesizer::new(
        &OPEN_JTALK.lock().await.0,
        InitializeOptions {
            acceleration_mode: if use_gpu {
                AccelerationMode::Gpu
            } else {
                AccelerationMode::Cpu
            },
            cpu_num_threads: cpu_num_threads as _,
        },
    )
    .unwrap();
    SYNTHESIZER.get_or_init(|| Mutex::new(SendSyncSynthesizer(synthesizer)));
}
pub static SYNTHESIZER: OnceLock<Mutex<SendSyncSynthesizer>> = OnceLock::new();

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsInitializedSpeakerQuery {
    speaker: u32,
}

pub async fn is_initialized_speaker_get(Query(query): Query<IsInitializedSpeakerQuery>) -> Result<Json<bool>> {
    let synthesizer = SYNTHESIZER.get().unwrap().lock().await;
    let metas = synthesizer.0.get_metas().map_err(anyhow::Error::from)?;
    Ok(Json(
        metas
            .iter()
            .any(|meta| meta.styles().iter().any(|m| m.id() == query.speaker)),
    ))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeSpeakerQuery {
    speaker: u32,
}

pub async fn initialize_speaker_post(Query(query): Query<InitializeSpeakerQuery>) -> Result<&'static str> {
    let vvm_manager = VVM_MANAGER.get().unwrap().lock().await;
    let synthesizer = SYNTHESIZER.get().unwrap().lock().await;

    for vvm in vvm_manager.vvms() {
        for meta in vvm.metas() {
            if meta.styles().iter().any(|m| m.id() == query.speaker) {
                synthesizer.0.load_voice_model(vvm).map_err(anyhow::Error::from)?;
                return Ok("");
            }
        }
    }

    Err(anyhow::anyhow!("指定されたスピーカーが見つかりませんでした。").into())
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioQueryQuery {
    text: String,
    speaker: u32,
    is_kana: Option<bool>,
}

pub async fn audio_query_post(Query(query): Query<AudioQueryQuery>) -> Result<Json<AudioQuery>> {
    let query = {
        let synthesizer = &SYNTHESIZER.get().unwrap().lock().await.0;
        if query.is_kana.unwrap_or(false) {
            synthesizer
                .create_audio_query_from_kana(&query.text, query.speaker)
                .map_err(anyhow::Error::from)?
        } else {
            synthesizer
                .create_audio_query(&query.text, query.speaker)
                .map_err(anyhow::Error::from)?
        }
    };
    let accent_phrases = serde_json::to_string(&query.accent_phrases).map_err(anyhow::Error::from)?;
    let accent_phrases: Vec<AccentPhrase> = serde_json::from_str(&accent_phrases).map_err(anyhow::Error::from)?;
    Ok(Json(AudioQuery {
        accent_phrases,
        speed_scale: query.speed_scale,
        pitch_scale: query.pitch_scale,
        intonation_scale: query.intonation_scale,
        volume_scale: query.volume_scale,
        pre_phoneme_length: query.pre_phoneme_length,
        post_phoneme_length: query.post_phoneme_length,
        output_sampling_rate: query.output_sampling_rate,
        output_stereo: query.output_stereo,
        kana: query.kana.unwrap_or_default(),
    }))
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccentPhraseQuery {
    text: String,
    speaker: u32,
    is_kana: Option<bool>,
}

pub async fn accent_phrases_post(
    Query(query): Query<AccentPhraseQuery>,
) -> Result<Json<Vec<voicevox_core_rs::AccentPhrase>>> {
    let query = {
        let synthesizer = &SYNTHESIZER.get().unwrap().lock().await.0;
        if query.is_kana.unwrap_or(false) {
            synthesizer
                .create_accent_phrases_from_kana(&query.text, query.speaker)
                .map_err(anyhow::Error::from)?
        } else {
            synthesizer
                .create_accent_phrases(&query.text, query.speaker)
                .map_err(anyhow::Error::from)?
        }
    };
    Ok(Json(query))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoraEditQuery {
    speaker: u32,
}

#[duplicate::duplicate_item(
     export_name        synthesizer_name;
    [mora_data_post]   [replace_mora_data];
    [mora_pitch_post]  [replace_mora_pitch];
    [mora_length_post] [replace_phoneme_length];
)]
pub async fn export_name(
    Query(query): Query<MoraEditQuery>,
    Json(accent_phrases): Json<Vec<voicevox_core_rs::AccentPhrase>>,
) -> Result<&'static str> {
    let synthesizer = &SYNTHESIZER.get().unwrap().lock().await.0;
    synthesizer
        .synthesizer_name(&accent_phrases, query.speaker)
        .map_err(anyhow::Error::from)?;
    Ok("")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynthesisQuery {
    speaker: u32,
    enable_interrogative_upspeak: bool,
}

pub async fn synthesis_post(
    Query(query): Query<SynthesisQuery>,
    Json(audio_query): Json<AudioQuery>,
) -> Result<Vec<u8>> {
    let accent_phrases = serde_json::to_string(&audio_query.accent_phrases).map_err(anyhow::Error::from)?;
    let accent_phrases: Vec<voicevox_core_rs::AccentPhrase> =
        serde_json::from_str(&accent_phrases).map_err(anyhow::Error::from)?;
    let audio = {
        let synthesizer = &SYNTHESIZER.get().unwrap().lock().await.0;
        synthesizer
            .synthesis(
                &voicevox_core_rs::AudioQuery {
                    accent_phrases,
                    speed_scale: audio_query.speed_scale,
                    pitch_scale: audio_query.pitch_scale,
                    intonation_scale: audio_query.intonation_scale,
                    volume_scale: audio_query.volume_scale,
                    pre_phoneme_length: audio_query.pre_phoneme_length,
                    post_phoneme_length: audio_query.post_phoneme_length,
                    output_sampling_rate: audio_query.output_sampling_rate,
                    output_stereo: audio_query.output_stereo,
                    kana: if audio_query.kana.is_empty() {
                        None
                    } else {
                        Some(audio_query.kana.clone())
                    },
                },
                query.speaker,
                SynthesisOptions {
                    enable_interrogative_upspeak: query.enable_interrogative_upspeak,
                },
            )
            .map_err(anyhow::Error::from)?
    };

    Ok(audio)
}
