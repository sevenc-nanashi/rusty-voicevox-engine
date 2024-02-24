use serde::{Deserialize, Serialize};
use std::sync::{Arc, OnceLock};
use tokio::sync::Mutex;
use tracing::info;
use voicevox_core_rs::{StyleMeta, StyleVersion, VoiceModel};

pub struct VvmManager {
    vvms: Vec<VoiceModel>,
    speakers: Vec<SpeakerMeta>,
}

unsafe impl Send for VvmManager {}
unsafe impl Sync for VvmManager {}

pub static VVM_MANAGER: OnceLock<Arc<Mutex<VvmManager>>> = OnceLock::new();

static VVM_DIR: &str = if cfg!(feature = "release") {
    "./vvms"
} else {
    concat!(env!("CARGO_MANIFEST_DIR"), "/vendored/voicevox_fat_resource/core/model")
};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SpeakerMeta {
    pub name: String,
    pub styles: Vec<StyleMeta>,
    pub version: StyleVersion,
    pub speaker_uuid: String,
}

impl VvmManager {
    pub async fn new() -> Self {
        let mut vvms = tokio::fs::read_dir(VVM_DIR).await.unwrap();

        info!("Loading VVMs...");
        let mut vvm_manager = VvmManager {
            vvms: vec![],
            speakers: vec![],
        };
        while let Some(vvm) = vvms.next_entry().await.unwrap() {
            let path = vvm.path();
            if path.is_file() && path.extension() == Some(std::ffi::OsStr::new("vvm")) {
                info!("  - {}", path.display());
                let vvm = VoiceModel::from_path(path).unwrap();
                vvm_manager.vvms.push(vvm);
            }
        }

        if vvm_manager.speakers.is_empty() {
            // TODO: 多分もっといい書き方があるけどとりあえずこれで動くのでヨシ！
            for vvm in &vvm_manager.vvms {
                for meta in vvm.metas() {
                    if vvm_manager
                        .speakers
                        .iter()
                        .any(|s: &SpeakerMeta| s.speaker_uuid == *meta.speaker_uuid())
                    {
                        dbg!(&meta.speaker_uuid());
                        let mut styles = vvm_manager
                            .speakers
                            .iter_mut()
                            .find(|s| s.speaker_uuid == *meta.speaker_uuid())
                            .unwrap()
                            .styles
                            .clone();

                        styles.extend(meta.styles().to_vec());

                        vvm_manager
                            .speakers
                            .iter_mut()
                            .find(|s| s.speaker_uuid == *meta.speaker_uuid())
                            .unwrap()
                            .styles = styles;
                    }

                    vvm_manager.speakers.push(meta.into());
                }
            }
        }

        vvm_manager
    }

    pub fn speakers(&self) -> &Vec<SpeakerMeta> {
        &self.speakers
    }

    pub fn speaker(&self, speaker_uuid: &str) -> Option<&SpeakerMeta> {
        self.speakers
            .iter()
            .find(|&speakers| speakers.speaker_uuid == speaker_uuid)
    }

    pub fn vvms(&self) -> &Vec<VoiceModel> {
        &self.vvms
    }
}

impl From<voicevox_core_rs::SpeakerMeta> for SpeakerMeta {
    fn from(speaker: voicevox_core_rs::SpeakerMeta) -> Self {
        SpeakerMeta {
            name: speaker.name().to_string(),
            styles: speaker.styles().to_vec(),
            version: speaker.version().to_owned(),
            speaker_uuid: speaker.speaker_uuid().to_string(),
        }
    }
}
