use crate::{result::Result, routes::synthesis::OPEN_JTALK};

use axum::{
    extract::{Path, Query},
    Json,
};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use tracing::warn;
use voicevox_core_rs::{UserDict, UserDictWord};

pub struct SendSyncUserDict(pub UserDict);

unsafe impl Send for SendSyncUserDict {}
unsafe impl Sync for SendSyncUserDict {}
static MORA_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(concat!(
        "(?:",
        "[イ][ェ]|[ヴ][ャュョ]|[トド][ゥ]|[テデ][ィャュョ]|[デ][ェ]|[クグ][ヮ]|", // rule_others
        "[キシチニヒミリギジビピ][ェャュョ]|",                                    // rule_line_i
        "[ツフヴ][ァ]|[ウスツフヴズ][ィ]|[ウツフヴ][ェォ]|",                      // rule_line_u
        "[ァ-ヴー]",                                                              // rule_one_mora
        ")",
    ))
    .unwrap()
});

pub static USER_DICT: Lazy<Arc<Mutex<SendSyncUserDict>>> = Lazy::new(|| {
    let user_dict = UserDict::new().unwrap();
    if std::fs::metadata(USER_DICT_PATH).is_ok() && user_dict.load(USER_DICT_PATH).is_err() {
        warn!("Failed to load user dict from {:?}", USER_DICT_PATH);
    }

    Arc::new(Mutex::new(SendSyncUserDict(user_dict)))
});

static USER_DICT_PATH: &str = "./user_dict.json";

#[derive(Debug, Serialize, Deserialize)]
pub struct VvUserDictWord {
    priority: u32,
    accent_type: usize,
    mora_count: usize,
    surface: String,
    pronunciation: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VvUserDictWordParam {
    priority: u32,
    accent_type: usize,
    surface: String,
    pronunciation: String,
}

impl From<VvUserDictWordParam> for UserDictWord {
    fn from(word: VvUserDictWordParam) -> UserDictWord {
        let mut w = UserDictWord::new(&word.surface[..], &word.pronunciation);
        w.accent_type = word.accent_type;
        w.priority = word.priority;
        w
    }
}

impl From<UserDictWord> for VvUserDictWord {
    fn from(word: UserDictWord) -> VvUserDictWord {
        VvUserDictWord {
            priority: word.priority,
            accent_type: word.accent_type,
            mora_count: MORA_REGEX.find_iter(&word.pronunciation).count(),
            surface: word.surface.clone(),
            pronunciation: word.pronunciation.clone(),
        }
    }
}
impl From<VvUserDictWord> for UserDictWord {
    fn from(word: VvUserDictWord) -> UserDictWord {
        let mut w = UserDictWord::new(&word.surface[..], &word.pronunciation);
        w.accent_type = word.accent_type;
        w.priority = word.priority;
        w
    }
}

pub async fn user_dict_get() -> Json<HashMap<String, VvUserDictWord>> {
    let user_dict = USER_DICT.lock().await;

    Json(serde_json::from_str(&serde_json::to_string(&user_dict.0).unwrap()).unwrap())
}

pub async fn import_user_dict_post(Json(payload): Json<HashMap<String, VvUserDictWord>>) -> Result<&'static str> {
    let temp_file = tempfile::NamedTempFile::new().map_err(anyhow::Error::from)?;

    let temp_file_writer = std::io::BufWriter::new(temp_file.as_file());

    serde_json::to_writer(temp_file_writer, &payload).map_err(anyhow::Error::from)?;

    let temp_file = temp_file.into_temp_path();

    tracing::debug!("Importing user dict from {:?}", temp_file);

    let user_dict = USER_DICT.lock().await;

    {
        let temp_user_dict = UserDict::new().map_err(anyhow::Error::from)?;
        temp_user_dict.load(&temp_file).map_err(anyhow::Error::from)?;

        user_dict.0.import(&temp_user_dict).map_err(anyhow::Error::from)?;
    }

    OPEN_JTALK.lock().await.0.use_user_dict(&user_dict.0).unwrap();

    Ok("")
}

pub async fn user_dict_word_post(Query(param): Query<VvUserDictWordParam>) -> Result<String> {
    let user_dict = &USER_DICT.lock().await;

    let word: UserDictWord = param.into();

    let word_uuid = user_dict.0.add_word(word).map_err(anyhow::Error::from)?;

    user_dict.0.save(USER_DICT_PATH).map_err(anyhow::Error::from)?;

    OPEN_JTALK.lock().await.0.use_user_dict(&user_dict.0).unwrap();

    Ok(word_uuid.hyphenated().to_string())
}

pub async fn user_dict_word_delete(Path(word_uuid): Path<String>) -> Result<&'static str> {
    let user_dict = &USER_DICT.lock().await;

    let word_uuid = uuid::Uuid::parse_str(&word_uuid).map_err(anyhow::Error::from)?;

    user_dict.0.remove_word(&word_uuid).map_err(anyhow::Error::from)?;

    user_dict.0.save(USER_DICT_PATH).map_err(anyhow::Error::from)?;

    OPEN_JTALK
        .lock()
        .await
        .0
        .use_user_dict(&user_dict.0)
        .map_err(anyhow::Error::from)?;

    Ok("")
}

pub async fn user_dict_word_put(
    Path(word_uuid): Path<String>,
    Query(payload): Query<VvUserDictWordParam>,
) -> Result<&'static str> {
    let user_dict = USER_DICT.lock().await;

    let word_uuid = uuid::Uuid::parse_str(&word_uuid).map_err(anyhow::Error::from)?;

    let word: UserDictWord = payload.into();

    user_dict.0.update_word(word_uuid, word).map_err(anyhow::Error::from)?;

    user_dict.0.save(USER_DICT_PATH).map_err(anyhow::Error::from)?;

    OPEN_JTALK
        .lock()
        .await
        .0
        .use_user_dict(&user_dict.0)
        .map_err(anyhow::Error::from)?;

    Ok("")
}
