use lyrix::models::TrackMetadata;
use lyrix::smtc_lyrics::{
    TOKEN,
    MusicPlayer,
    get_lyrics_with_player,
    get_trial_part,
};

#[allow(unused_variables)]
fn jtrack(s: &str) -> TrackMetadata {
    TrackMetadata {
        title: Some("メルト (Melt) (CPK! Remix|かぐや ver.)".to_string()),
        artist: Some(format!("ryo {} 夏吉ゆうこ", s)),
        album: Some("超かぐや姫！".to_string()),
        album_artist: Some("超かぐや姫！".to_string()),
        duration_ms: Some(271627),
        ..Default::default()
    }
}

#[allow(unused_variables)]
fn etrack(s: &str) -> TrackMetadata {
    TrackMetadata {
        title: Some("Is There Someone Else?".to_string()),
        artist: Some(format!("The Weeknd")),
        album: Some("".to_string()),
        album_artist: Some("".to_string()),
        duration_ms: Some(60055u32),
        ..Default::default()
    }
}

#[allow(unused)]
fn ttrack(s: &str) -> TrackMetadata {
    TrackMetadata {
        title: Some("Extraordinary".to_string()),
        artist: Some(format!("Connor Price")),
        album: Some("".to_string()),
        album_artist: Some("".to_string()),
        duration_ms: None,
        ..Default::default()
    }
}

#[tokio::test]
async fn test_apple_music_normal() {
    *TOKEN.lock().unwrap() = "自己填自己的".to_string();
    let a = "小糸 侑(CV:高田憂希)、七海燈子(CV:寿 美菜子) — TVアニメ「やがて君になる」エンディングテーマ「hectopascal」 - EP".to_string();
    let result = get_lyrics_with_player(
        &MusicPlayer::AppleMusic,
        "hectopascal",
        Some(&a),
        Some(&a),
        Some(&a),
        237507,
    ).await;
    println!("{:?}", result)
}

#[tokio::test]
async fn test_apple_music() {
    *TOKEN.lock().unwrap() = "自己填自己的".to_string();
    let a = "Meg Myers — Running Up That Hill - Single".to_string();
    let result = get_lyrics_with_player(
        &MusicPlayer::AppleMusic,
        "Running Up That Hill",
        Some(&a),
        Some(&a),
        Some(&a),
        263717,
    ).await;
    println!("{:?}", result)
}

#[tokio::test]
async fn test_netease() {
    let track = etrack("/");
    let result = get_lyrics_with_player(
        &MusicPlayer::Netease,
        track.title.as_deref().unwrap(),
        track.artist.as_deref(),
        track.album.as_deref(),
        track.album_artist.as_deref(),
        track.duration_ms.unwrap(),
    ).await;
    println!("{:?}", &result);
    let n = get_trial_part(result.unwrap());
    println!("{:?}", n);
}

#[tokio::test]
async fn test_qqmusic() {
    let track = etrack("/");
    let result = get_lyrics_with_player(
        &MusicPlayer::QQMusic,
        track.title.as_deref().unwrap(),
        track.artist.as_deref(),
        track.album.as_deref(),
        track.album_artist.as_deref(),
        track.duration_ms.unwrap(),
    ).await;
    println!("{:?}", result);
    let n = get_trial_part(result.unwrap());
    println!("{:?}", n);
}

#[tokio::test]
async fn test_kugou_music() {
    let track = jtrack("、");
    let result = get_lyrics_with_player(
        &MusicPlayer::Kugou,
        track.title.as_deref().unwrap(),
        track.artist.as_deref(),
        track.album.as_deref(),
        track.album_artist.as_deref(),
        track.duration_ms.unwrap(),
    ).await;
    println!("{:?}", result)
}

#[tokio::test]
async fn test_soda_music() {
    let result = get_lyrics_with_player(
        &MusicPlayer::SodaMusic,
        "Destiny",
        Some("AG710X"),
        Some(""),
        Some(""),
        126199,
    ).await;
    println!("{:?}", result)
}
