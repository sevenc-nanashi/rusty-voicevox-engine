use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioQuery {
    #[serde(rename = "accent_phrases")]
    pub accent_phrases: Vec<AccentPhrase>,
    pub speed_scale: f32,
    pub pitch_scale: f32,
    pub intonation_scale: f32,
    pub volume_scale: f32,
    pub pre_phoneme_length: f32,
    pub post_phoneme_length: f32,
    pub output_sampling_rate: u32,
    pub output_stereo: bool,
    #[serde(default)]
    pub kana: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccentPhrase {
    pub moras: Vec<Mora>,
    pub accent: u32,
    pub pause_mora: Option<Mora>,
    pub is_interrogative: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Mora {
    pub text: String,
    pub consonant: Option<String>,
    pub consonant_length: Option<f32>,
    pub vowel: String,
    pub vowel_length: f32,
    pub pitch: f32,
}
