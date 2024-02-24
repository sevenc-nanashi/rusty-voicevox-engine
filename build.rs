use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize)]
struct SpeakerColor {
    color: (u8, u8, u8),
    light_color: (u8, u8, u8),
}

static MPLUS_PATH: &str = "vendored/mplus-2p-semibold.ttf";
static SPEAKER_COLORS_PATH: &str = "vendored/speaker_colors.json";
static OPEN_JTALK_DICT_PATH: &str = "vendored/open_jtalk_dic_utf_8-1.11";

fn download_dict() {
    if (std::fs::metadata(OPEN_JTALK_DICT_PATH)).is_ok() {
        return;
    }

    let tar_gz = ureq::get("https://jaist.dl.sourceforge.net/project/open-jtalk/Dictionary/open_jtalk_dic-1.11/open_jtalk_dic_utf_8-1.11.tar.gz")
    .call()
    .unwrap()
    .into_reader();

    let tar = flate2::read::GzDecoder::new(tar_gz);
    let mut archive = tar::Archive::new(tar);
    let dest_path = std::path::Path::new(OPEN_JTALK_DICT_PATH);
    for mut entry in archive.entries().unwrap().filter_map(|e| e.ok()) {
        let path = entry.path().unwrap();
        let path = path.strip_prefix("open_jtalk_dic_utf_8-1.11/").unwrap();
        entry.unpack(dest_path.join(path)).unwrap();
    }
}
fn download_mplus() {
    if (std::fs::metadata(MPLUS_PATH)).is_ok() {
        return;
    }
    let resp = ureq::get("https://github.com/coz-m/MPLUS_FONTS/raw/master/fonts/ttf/Mplus2-SemiBold.ttf")
        .call()
        .unwrap();
    let mut file = std::fs::File::create(MPLUS_PATH).unwrap();
    let mut reader = std::io::BufReader::new(resp.into_reader());
    std::io::copy(&mut reader, &mut file).unwrap();
}
fn between<'a>(s: &'a str, start: &str, end: &str) -> Option<&'a str> {
    let start = s.find(start)? + start.len();
    let end = s[start..].find(end)? + start;
    Some(&s[start..end])
}

fn scrape_speaker_colors() {
    if (std::fs::metadata(SPEAKER_COLORS_PATH)).is_ok() {
        return;
    }
    let name_map = {
        let name_body = ureq::get(
        "https://raw.githubusercontent.com/VOICEVOX/voicevox_blog/f0999b7856554ff45383be242ac2f64613ce5888/src/constants.ts",
    ).call().unwrap().into_string().unwrap();
        let name_body = name_body[name_body.find("export const characterInfos: {").unwrap()..].to_string();
        let mut name_map: HashMap<String, String> = HashMap::new();
        let mut current_character = String::new();
        for line in name_body.lines() {
            if line.starts_with("  ") && line.contains(": {") {
                current_character = line.trim().trim_end_matches(": {").to_string();
            } else if line.starts_with("    name: ") {
                let name = line
                    .trim()
                    .trim_end_matches(',')
                    .trim_start_matches("name: ")
                    .trim_matches('"');
                if current_character.is_empty() {
                    panic!("Failed to parse character info");
                }
                name_map.insert(current_character.clone(), name.to_string());
            }
        }
        name_map
    };
    let character_colors = {
        let color_body = ureq::get(
            "https://raw.githubusercontent.com/VOICEVOX/voicevox_blog/master/src/hooks/useDetailedCharacterInfo.ts",
        )
        .call()
        .unwrap()
        .into_string()
        .unwrap();
        let character_infos = between(&color_body, "  const _characterInfos: {", "} as const").unwrap();
        let mut current_character = String::new();
        let mut current_color = String::new();
        let mut character_colors: HashMap<String, SpeakerColor> = HashMap::new();
        for line in character_infos.lines() {
            if line.starts_with("    ") && line.contains(": {") {
                current_character = line.trim().trim_end_matches(": {").to_string();
                dbg!(&current_character);
            } else if line.starts_with("      color: ") {
                let color = line
                    .trim()
                    .trim_end_matches(',')
                    .trim_start_matches("color: ")
                    .trim_matches('"');
                current_color = color.to_string();
            } else if line.starts_with("      lightColor: ") {
                let light_color = line
                    .trim()
                    .trim_end_matches(',')
                    .trim_start_matches("lightColor: ")
                    .trim_matches('"');
                if current_character.is_empty() || current_color.is_empty() {
                    panic!("Failed to parse character info");
                }
                character_colors.insert(
                    name_map.get(&current_character).unwrap().to_string(),
                    SpeakerColor {
                        color: to_rgb(&current_color),
                        light_color: to_rgb(light_color),
                    },
                );
                current_character.clear();
                current_color.clear();
            }
        }
        character_colors
    };

    let mut file = std::fs::File::create(SPEAKER_COLORS_PATH).unwrap();
    serde_json::to_writer_pretty(&mut file, &character_colors).unwrap();
}

fn main() {
    download_mplus();
    scrape_speaker_colors();
    download_dict();
}

fn to_rgb(hex: &str) -> (u8, u8, u8) {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap();
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap();
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap();
    (r, g, b)
}
