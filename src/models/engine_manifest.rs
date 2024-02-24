use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EngineManifest {
    pub manifest_version: String,
    pub name: String,
    pub brand_name: String,
    pub uuid: String,
    pub url: String,
    pub icon: String,
    pub default_sampling_rate: i64,
    pub terms_of_service: String,
    pub update_infos: Vec<UpdateInfo>,
    pub dependency_licenses: Vec<DependencyLicense>,
    pub supported_features: SupportedFeatures,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub version: String,
    pub descriptions: Vec<String>,
    pub contributors: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DependencyLicense {
    pub name: String,
    pub version: String,
    pub license: String,
    pub text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SupportedFeatures {
    pub adjust_mora_pitch: bool,
    pub adjust_phoneme_length: bool,
    pub adjust_speed_scale: bool,
    pub adjust_pitch_scale: bool,
    pub adjust_intonation_scale: bool,
    pub adjust_volume_scale: bool,
    pub interrogative_upspeak: bool,
    pub synthesis_morphing: bool,
    pub manage_library: bool,
}
