use dialoguer::{theme::ColorfulTheme, Input, Select};
use lyrix::smtc_lyrics::{Session, MusicPlayer, get_lyrics_with_player};

const APP_IDS: &[(&str, MusicPlayer)] = &[
    ("cloudmusic.exe",                                MusicPlayer::Netease),
    ("qqmusic.exe",                                   MusicPlayer::QQMusic),
    ("kugou",                                         MusicPlayer::Kugou),
    ("\u{6c7d}\u{6c34}\u{97f3}\u{4e50}",              MusicPlayer::SodaMusic),
    ("AppleInc.AppleMusicWin_nzyj5cx40ttqa!App",      MusicPlayer::AppleMusic),
    ("Spotify.exe",      MusicPlayer::Spotify),
];

fn player_json_key(player: MusicPlayer) -> &'static str {
    match player {
        MusicPlayer::Netease => "netease",
        MusicPlayer::QQMusic => "qqmusic",
        MusicPlayer::Kugou => "kugou",
        MusicPlayer::SodaMusic => "soda_music",
        MusicPlayer::AppleMusic => "applemusic",
        MusicPlayer::Spotify => "spotify",
        _ => {unreachable!("unsupported player")},
    }
}

#[tokio::test]
async fn test_interactive() {
    let track_db: Option<serde_json::Value> =
        std::fs::read_to_string("tests/track.json")
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok());

    let labels: Vec<&str> = APP_IDS.iter().map(|(_, p)| p.display_name()).collect();
    let sel = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("选择播放器")
        .items(&labels)
        .default(0)
        .interact()
        .unwrap();
    let (_app_id, player) = APP_IDS[sel];


    let trial_labels = &["非试听 (ntrial)", "试听 (trial)"];
    let trial_sel = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("选择模式")
        .items(trial_labels)
        .default(0)
        .interact()
        .unwrap();
    let trial_key = if trial_sel == 0 { "ntrial" } else { "trial" };


    let mut track_keys: Vec<String> = Vec::new();
    let mut track_labels: Vec<String> = Vec::new();
    if let Some(ref db) = track_db {
        if let Some(tracks) = db.get(player_json_key(player))
            .and_then(|p| p.get(trial_key))
        {
            if let Some(obj) = tracks.as_object() {
                for (key, val) in obj {
                    let title = val.get("title").and_then(|v| v.as_str()).unwrap_or(key);
                    track_labels.push(format!("{} ({})", key, title));
                    track_keys.push(key.clone());
                }
            }
        }
    }

    track_labels.push("手动输入".to_string());

    let track_sel = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("选择曲目")
        .items(&track_labels)
        .default(0)
        .interact()
        .unwrap();


    let (title, artist, album, album_artist, duration_ms) = if track_sel < track_keys.len() {
        let track_key = &track_keys[track_sel];

        // 从 track.json 读取（此时一定能读到，因为 track_keys 就是从 JSON 构建的）
        if let Some(ref db) = track_db {
            if let Some(track_data) = db.get(player_json_key(player))
                .and_then(|t| t.get(trial_key))
                .and_then(|t| t.get(track_key))
            {
                let t = track_data;
                (
                    t.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    t.get("artist").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    t.get("album").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    t.get("album_artist").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    t.get("duration_ms").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                )
            } else {
                unreachable!()
            }
        } else {
            unreachable!()
        }
    } else {
        let title: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("title")
            .interact_text()
            .unwrap();
        let artist: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("artist (可空)")
            .allow_empty(true)
            .interact_text()
            .unwrap();
        let album: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("album (可空)")
            .allow_empty(true)
            .interact_text()
            .unwrap();
        let duration_ms: u32 = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("duration_ms")
            .default(0u32)
            .interact_text()
            .unwrap();
        (title, artist, album, String::new(), duration_ms)
    };

    let mut applemusic_token = None;
    let mut spotify_cookie = None;
    //json在外面你气不气
    if let Ok(content) = std::fs::read_to_string("../auth.json") {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            applemusic_token = json.get("applemusic_token").and_then(|v| v.as_str().map(String::from));
            spotify_cookie = json.get("spotify_cookie").and_then(|v| v.as_str().map(String::from));
        }
    }
    let mut session = Session {
        applemusic_token,
        spotify_cookie,
    };
    if player == MusicPlayer::AppleMusic && session.applemusic_token.is_none() {
        let token: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Apple Music token")
            .interact_text()
            .unwrap();
        session.applemusic_token = Some(token);
    }

    let artist_opt = (!artist.is_empty()).then_some(artist.as_str());
    let album_opt = (!album.is_empty()).then_some(album.as_str());
    let album_artist_opt = (!album_artist.is_empty()).then_some(album_artist.as_str());

    let result = get_lyrics_with_player(
        &player,
        &title,
        artist_opt,
        album_opt,
        album_artist_opt,
        duration_ms,
        &session,
    )
    .await;

    match &result {
        Ok(data) => {
            println!("track_metadata: {:?}", &data.track_metadata);
            println!("\n=== {} lines ===", data.lines.len());
            for (_i, line) in data.lines.iter().enumerate().take(1) {
                if line.syllables.is_empty() {
                    println!("[{}ms] {}", line.start_time, line.text);
                } else {
                    for word in &line.syllables {
                        println!("[{}ms] {}", word.start_time, word.text);
                    }
                }
            }
        }
        Err(e) => println!("\nError: {}", e),
    }
}
