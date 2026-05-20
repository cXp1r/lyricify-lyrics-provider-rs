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

fn split_char(player: MusicPlayer) -> &'static str {
    match player {
        MusicPlayer::Kugou => "、",
        MusicPlayer::Netease | MusicPlayer::QQMusic => "/",
        MusicPlayer::SodaMusic => ",",
        MusicPlayer::AppleMusic => " ",
        MusicPlayer::Spotify => " ",//实则只给第一个艺人
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

fn ttrack() -> TrackMetadata {
    TrackMetadata {
        title: Some("Extraordinary".to_string()),
        artist: Some("Connor Price".to_string()),
        album: Some("".to_string()),
        album_artist: Some("".to_string()),
        duration_ms: None,
        ..Default::default()
    }
}

#[tokio::test]
async fn test_interactive() {
    // 1. 选播放器
    let labels: Vec<&str> = APP_IDS.iter().map(|(_, p)| p.display_name()).collect();
    let sel = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("选择播放器")
        .items(&labels)
        .default(0)
        .interact()
        .unwrap();
    let (_app_id, player) = APP_IDS[sel];

    // 2. 选曲目
    let preset_labels = &[
        "jtrack (メルト)",
        "etrack (Is There Someone Else?)",
        "ttrack (Extraordinary)",
        "手动输入",
    ];
    let preset_sel = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("选择曲目")
        .items(preset_labels)
        .default(0)
        .interact()
        .unwrap();

    let (title, artist, album, album_artist, duration_ms) = if preset_sel < 3 {
        let track = match preset_sel {
            0 => jtrack(player),
            1 => etrack(),
            2 => ttrack(),
            _ => unreachable!(),
        };
        (
            track.title.unwrap(),
            track.artist.unwrap(),
            track.album.unwrap(),
            track.album_artist.unwrap(),
            track.duration_ms.unwrap_or(0),
        )
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

    // 3. Session
    let mut session = Session {
        applemusic_token: None,
        spotify_cookie: None,
    };
    if player == MusicPlayer::AppleMusic {
        let token: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Apple Music token")
            .interact_text()
            .unwrap();
        session.applemusic_token = Some(token);
    }

    let artist_opt = (!artist.is_empty()).then_some(artist.as_str());
    let album_opt = (!album.is_empty()).then_some(album.as_str());
    let album_artist_opt = (!album_artist.is_empty()).then_some(album_artist.as_str());

    // 4. 调用
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
