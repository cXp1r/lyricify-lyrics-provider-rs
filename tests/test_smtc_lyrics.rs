use dialoguer::{theme::ColorfulTheme, Input, Select};
use lyrix::models::TrackMetadata;
use lyrix::smtc_lyrics::{Session, MusicPlayer, get_lyrics_with_player, get_trial_part};

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
    }
}

fn split_char(player: MusicPlayer) -> &'static str {
    match player {
        MusicPlayer::Kugou => "、",
        MusicPlayer::Netease | MusicPlayer::QQMusic => "/",
        MusicPlayer::SodaMusic => ",",
        MusicPlayer::AppleMusic => " ",
        MusicPlayer::Spotify => " ",
    }
}

fn jtrack(player: MusicPlayer) -> TrackMetadata {
    TrackMetadata {
        title: Some("メルト (Melt) (CPK! Remix|かぐや ver.)".to_string()),
        artist: Some(format!("ryo {} 夏吉ゆうこ", split_char(player))),
        album: Some("超かぐや姫！".to_string()),
        album_artist: Some("超かぐや姫！".to_string()),
        duration_ms: Some(271627),
        ..Default::default()
    }
}

fn etrack() -> TrackMetadata {
    TrackMetadata {
        title: Some("Is There Someone Else?".to_string()),
        artist: Some("The Weeknd".to_string()),
        album: Some("".to_string()),
        album_artist: Some("".to_string()),
        duration_ms: Some(60055u32),
        ..Default::default()
    }
}

fn ctrack() -> TrackMetadata {
    TrackMetadata {
        title: Some("弱水三千".to_string()),
        artist: Some("石头/张晓棠".to_string()),
        album: Some("念".to_string()),
        album_artist: Some("".to_string()),
        duration_ms: None,
        ..Default::default()
    }
}

#[tokio::test]
async fn test_interactive() {
    // 加载 track.json
    let track_db: Option<serde_json::Value> =
        std::fs::read_to_string("tests/track.json")
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok());

    // 1. 选试听模式
    let trial_labels = &["非试听 (ntrial)", "试听 (trial)"];
    let trial_sel = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("选择模式")
        .items(trial_labels)
        .default(0)
        .interact()
        .unwrap();
    let trial_key = if trial_sel == 0 { "ntrial" } else { "trial" };

    // 2. 选曲目 (j/e/c)
    let track_labels = &[
        "jtrack (メルト / rise)",
        "etrack (Is There Someone Else? / After Hours)",
        "ctrack (弱水三千 / 告白气球)",
        "手动输入",
    ];
    let track_sel = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("选择曲目")
        .items(track_labels)
        .default(0)
        .interact()
        .unwrap();

    // 3. 选播放器
    let labels: Vec<&str> = APP_IDS.iter().map(|(_, p)| p.display_name()).collect();
    let sel = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("选择播放器")
        .items(&labels)
        .default(0)
        .interact()
        .unwrap();
    let (_app_id, player) = APP_IDS[sel];

    // 4. 获取曲目元数据：优先从 track.json 查找，其次用硬编码回退
    let (title, artist, album, album_artist, duration_ms) = if track_sel < 3 {
        let track_key = match track_sel {
            0 => "jtrack",
            1 => "etrack",
            2 => "ctrack",
            _ => unreachable!(),
        };

        // 尝试从 track.json 读取
        if let Some(ref db) = track_db {
            if let Some(track_data) = db.get(trial_key)
                .and_then(|t| t.get(track_key))
                .and_then(|t| t.get(player_json_key(player)))
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
                // track.json 无此条目，用硬编码回退
                let track = match track_sel {
                    0 => jtrack(player),
                    1 => etrack(),
                    2 => ctrack(),
                    _ => unreachable!(),
                };
                (
                    track.title.unwrap(),
                    track.artist.unwrap(),
                    track.album.unwrap(),
                    track.album_artist.unwrap(),
                    track.duration_ms.unwrap_or(0),
                )
            }
        } else {
            // track.json 不存在，用硬编码回退
            let track = match track_sel {
                0 => jtrack(player),
                1 => etrack(),
                2 => ctrack(),
                _ => unreachable!(),
            };
            (
                track.title.unwrap(),
                track.artist.unwrap(),
                track.album.unwrap(),
                track.album_artist.unwrap(),
                track.duration_ms.unwrap_or(0),
            )
        }
    } else {
        // 手动输入
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
            println!("\n=== {} lines ===", data.lines.len());
            for line in &data.lines {
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

    if let Ok(data) = result {
        if let Ok(trial) = get_trial_part(data) {
            println!("\n--- trial ({} lines) ---", trial.lines.len());
            for line in &trial.lines {
                let text: String = if line.syllables.is_empty() {
                    line.text.clone()
                } else {
                    line.syllables.iter().map(|s| s.text.as_str()).collect::<Vec<_>>().join("")
                };
                println!("[{}ms] {}", line.start_time, text);
            }
        }
    }
}
