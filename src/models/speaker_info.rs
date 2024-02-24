use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpeakerInfo {
    pub policy: String,
    pub portrait: String,
    pub style_infos: Vec<StyleInfo>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StyleInfo {
    pub id: i64,
    pub icon: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub portrait: Option<String>,
    pub voice_samples: Vec<String>,
}
