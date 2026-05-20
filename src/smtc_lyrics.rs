use async_trait::async_trait;

use crate::models::{LineInfo, LyricsData, TrackMetadata, ITrackMetadata};
use crate::searchers::{ISearcher, ISearchResult};

pub struct Session {
    pub applemusic_token: Option<String>,
    pub spotify_cookie: Option<String>,
}
// ===== MusicPlayer =====

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MusicPlayer {
    Kugou,
    Netease,
    QQMusic,
    SodaMusic,
    Spotify,
    AppleMusic,
}

impl MusicPlayer {
    pub fn display_name(&self) -> &str {
        match self {
            MusicPlayer::Kugou => "酷狗音乐",
            MusicPlayer::Netease => "网易云音乐",
            MusicPlayer::QQMusic => "QQ音乐",
            MusicPlayer::SodaMusic => "汽水音乐",
            MusicPlayer::Spotify => "Spotify",
            MusicPlayer::AppleMusic => "AppleMusic",
        }
    }

    pub fn all_sorted() -> &'static [MusicPlayer] {
        &[
            MusicPlayer::Kugou,
            MusicPlayer::Netease,
            MusicPlayer::QQMusic,
            MusicPlayer::SodaMusic,
            MusicPlayer::AppleMusic,
            MusicPlayer::Spotify,
        ]
    }
}

pub fn id2player(app_id: &str) -> Result<MusicPlayer, String> {
    Ok(match app_id {
        "cloudmusic.exe" => MusicPlayer::Netease,
        "qqmusic.exe" => MusicPlayer::QQMusic,
        "kugou" => MusicPlayer::Kugou,
        "\u{6c7d}\u{6c34}\u{97f3}\u{4e50}" => MusicPlayer::SodaMusic,
        "AppleInc.AppleMusicWin_nzyj5cx40ttqa!App" => MusicPlayer::AppleMusic,
        "Spotify.exe" => MusicPlayer::Spotify,
        _ => return Err(format!("Unsupported appid: {}", app_id)),
    })
}

// ===== 公开接口 =====

pub async fn get_lyrics_with_player(
    player: &MusicPlayer,
    title: &str,
    artist: Option<&str>,
    album: Option<&str>,
    album_artist: Option<&str>,
    duration_ms: u32,
    session: &Session,
) -> Result<LyricsData, Box<dyn std::error::Error + Send + Sync>> {
    let metadata = TrackMetadata {
        title: Some(title.to_string()),
        artist: artist.map(|s| s.to_string()),
        album: album.map(|s| s.to_string()),
        album_artist: album_artist.map(|s| s.to_string()),
        duration_ms: Some(duration_ms),
        ..Default::default()
    };
    fetch_lyrics_from_player(player, &metadata, session).await
}

pub async fn get_lyrics_with_appid(
    app_id: &str,
    title: &str,
    artist: Option<&str>,
    album: Option<&str>,
    album_artist: Option<&str>,
    duration_ms: u32,
    session: &Session,
) -> Result<LyricsData, Box<dyn std::error::Error + Send + Sync>> {
    let player = id2player(app_id)?;
    let metadata = TrackMetadata {
        title: Some(title.to_string()),
        artist: artist.map(|s| s.to_string()),
        album: album.map(|s| s.to_string()),
        album_artist: album_artist.map(|s| s.to_string()),
        duration_ms: Some(duration_ms),
        ..Default::default()
    };
    fetch_lyrics_from_player(&player, &metadata, session).await
}

pub fn get_trial_part(raw: LyricsData) -> Result<LyricsData, String> {
    let (st, du) = match &raw.track_metadata {
        Some(op) => match &op.trial {
            Some(trial) => (trial[0], trial[1]),
            None => return Err("cannot find trial info".into()),
        },
        None => return Err("cannot find track_metadata".into()),
    };
    let raw_lines= raw.lines;
    let mut new_lines: Vec<LineInfo> = Vec::new();
    for x in raw_lines {
        if x.start_time < st {
            continue;
        }
        if x.start_time > st + du {
            break;
        }
        new_lines.push(LineInfo { start_time: x.start_time - st, ..x });
    }
    Ok(LyricsData { lines: new_lines, ..raw })
}

///严肃采用trait
#[async_trait]
trait LyricsProvider {
    type Searcher: ISearcher;
    type Api: Send + Sync;
    type SearchResult: ISearchResult + 'static;

    fn create_searcher(&self) -> Self::Searcher;
    fn create_api(&self) -> Self::Api;
    fn label() -> &'static str;
    async fn fetch_and_parse(
        api: &Self::Api,
        best: &Self::SearchResult,
    ) -> Result<Vec<LineInfo>, Box<dyn std::error::Error + Send + Sync>>;
}

///通用函数
async fn fetch_lyrics<P: LyricsProvider>(
    provider: &P,
    track: &dyn ITrackMetadata,
) -> Result<LyricsData, Box<dyn std::error::Error + Send + Sync>> {
    let searcher = provider.create_searcher();
    let result = searcher
        .search_for_result(track)
        .await?
        .ok_or_else(|| format!("{}: 未找到匹配的歌曲", P::label()))?;

    let best = result
        .as_any()
        .downcast_ref::<P::SearchResult>()
        .ok_or_else(|| format!("{}: 搜索结果类型不匹配", P::label()))?;

    let api = provider.create_api();
    let lines = P::fetch_and_parse(&api, best).await?;

    if lines.is_empty() {
        return Err(format!("{}: 未获取到歌词内容", P::label()).into());
    }

    Ok(LyricsData {
        file: None,
        lines,
        track_metadata: Some(TrackMetadata {
            title: Some(best.title().to_string()),
            artist: Some(best.artists().join(", ")),
            album: Some(best.album().to_string()),
            duration_ms: best.duration_ms(),
            score: best.match_score(),
            is_trial: best.trial().is_some(),
            trial: best.trial(),
            ..Default::default()
        }),
    })
}

//本质分发
async fn fetch_lyrics_from_player(
    player: &MusicPlayer,
    track: &dyn ITrackMetadata,
    session: &Session,
) -> Result<LyricsData, Box<dyn std::error::Error + Send + Sync>> {
    match player {
        MusicPlayer::Netease => fetch_lyrics(&NeteaseProvider, track).await,
        MusicPlayer::QQMusic => fetch_lyrics(&QQMusicProvider, track).await,
        MusicPlayer::Kugou => fetch_lyrics(&KugouProvider, track).await,
        MusicPlayer::SodaMusic => fetch_lyrics(&SodaMusicProvider, track).await,
        MusicPlayer::Spotify => {
            let cookie = session
                .spotify_cookie
                .as_ref()
                .ok_or("Spotify token not set")?
                .clone();
            fetch_lyrics(&SpotifyProvider { cookie }, track).await
        },
        MusicPlayer::AppleMusic => {
            let token = session
                .applemusic_token
                .as_ref()
                .ok_or("Apple Music token not set")?
                .clone();
            fetch_lyrics(&AppleMusicProvider { token }, track).await
        }

    }
}



struct NeteaseProvider;
struct QQMusicProvider;
struct KugouProvider;
struct SodaMusicProvider;
struct AppleMusicProvider {
    token: String,
}
struct SpotifyProvider {
    cookie: String,
}

#[async_trait]
impl LyricsProvider for NeteaseProvider {
    type Searcher = crate::searchers::netease::NeteaseSearcher;
    type Api = crate::providers::netease::NeteaseApi;
    type SearchResult = crate::searchers::netease::NeteaseSearchResult;

    fn create_searcher(&self) -> Self::Searcher {
        crate::searchers::netease::NeteaseSearcher::new()
    }
    fn create_api(&self) -> Self::Api {
        crate::providers::netease::NeteaseApi::new()
    }
    fn label() -> &'static str {
        "网易云"
    }

    async fn fetch_and_parse(
        api: &Self::Api,
        best: &Self::SearchResult,
    ) -> Result<Vec<LineInfo>, Box<dyn std::error::Error + Send + Sync>> {
        use crate::parsers::netease::{NeteaseParser, NeteaseLrcParser};
        use crate::parsers::IParsers;
        use crate::parsers::lrc::LrcParser;
        let lyric_result = api.get_lyric(&best.id).await?;
        if let Some(yrc) = lyric_result.yrc.and_then(|y| y.lyric) {
            if !yrc.is_empty() {
                return Ok(NeteaseParser {}.parse(yrc)?);
            }
        }
        let lrc = lyric_result.lrc.ok_or("网易云: LRC也没有哟")?;
        let parser = NeteaseLrcParser { version: lrc.version.unwrap_or(3) as u8 };
        Ok(parser.parse(lrc.lyric.ok_or("网易云: LRC也没有哟")?)?)
    }
}

#[async_trait]
impl LyricsProvider for QQMusicProvider {
    type Searcher = crate::searchers::qqmusic::QQMusicSearcher;
    type Api = crate::providers::qqmusic::QQMusicApi;
    type SearchResult = crate::searchers::qqmusic::QQMusicSearchResult;

    fn create_searcher(&self) -> Self::Searcher {
        crate::searchers::qqmusic::QQMusicSearcher::new()
    }
    fn create_api(&self) -> Self::Api {
        crate::providers::qqmusic::QQMusicApi::new()
    }
    fn label() -> &'static str {
        "QQ音乐"
    }

    async fn fetch_and_parse(
        api: &Self::Api,
        best: &Self::SearchResult,
    ) -> Result<Vec<LineInfo>, Box<dyn std::error::Error + Send + Sync>> {
        use crate::parsers::qqmusic::{QQMusicParser, QQMusicLrcParser};
        use crate::parsers::lrc::LrcParser;
        if let Ok(qrc) = api.get_lyrics_qrc(&best.id.to_string()).await {
            return Ok(QQMusicParser {}.decrypt_and_parse(qrc)?);
        }
        let lyric_result = api
            .get_lyric(&best.mid)
            .await?
            .ok_or("QQ音乐: 获取歌词失败")?;
        if let Some(lrc) = lyric_result.lyric {
            if !lrc.is_empty() {
                return Ok(QQMusicLrcParser {}.parse(lrc)?);
            }
        }
        Err("QQ音乐: 未获取到歌词内容".into())
    }
}

#[async_trait]
impl LyricsProvider for KugouProvider {
    type Searcher = crate::searchers::kugou::KugouSearcher;
    type Api = crate::providers::kugou::KugouApi;
    type SearchResult = crate::searchers::kugou::KugouSearchResult;

    fn create_searcher(&self) -> Self::Searcher {
        crate::searchers::kugou::KugouSearcher::new()
    }
    fn create_api(&self) -> Self::Api {
        crate::providers::kugou::KugouApi::new()
    }
    fn label() -> &'static str {
        "酷狗"
    }

    async fn fetch_and_parse(
        api: &Self::Api,
        best: &Self::SearchResult,
    ) -> Result<Vec<LineInfo>, Box<dyn std::error::Error + Send + Sync>> {
        use crate::parsers::kugou::KugouParser;
        let keyword = format!("{} {}", best.title, best.artists.join(", "));
        let lyrics_resp = api
            .get_search_lyrics(Some(&keyword), Some(&best.hash))
            .await?
            .ok_or("酷狗: 获取歌词候选失败")?;
        let candidates = lyrics_resp.candidates.unwrap_or_default();
        let candidate = candidates.first().ok_or("酷狗: 无歌词候选")?;
        let id = candidate.id.as_deref().ok_or("酷狗: 候选缺少 id")?;
        let access_key = candidate.access_key.as_deref().ok_or("酷狗: 候选缺少 accesskey")?;
        let dl_resp = api
            .get_download_krc(id, access_key)
            .await?
            .ok_or("酷狗: 下载 KRC 失败")?;
        let krc = dl_resp.content.ok_or("酷狗: KRC 内容为空")?;
        if krc.is_empty() {
            return Err("酷狗: KRC 内容为空".into());
        }
        Ok(KugouParser {}.decrypt_and_parse(krc)?)
    }
}

#[async_trait]
impl LyricsProvider for SpotifyProvider {
    type Searcher = crate::searchers::spotify::SpotifySearcher;
    type Api = crate::providers::spotify::SpotifyApi;
    type SearchResult = crate::searchers::spotify::SpotifySearchResult;

    fn create_searcher(&self) -> Self::Searcher {
        crate::searchers::spotify::SpotifySearcher::new(self.cookie.clone())
    }
    fn create_api(&self) -> Self::Api {
        crate::providers::spotify::SpotifyApi::new(self.cookie.clone())
    }
    fn label() -> &'static str {
        "汽水音乐"
    }

    async fn fetch_and_parse(
        api: &Self::Api,
        best: &Self::SearchResult,
    ) -> Result<Vec<LineInfo>, Box<dyn std::error::Error + Send + Sync>> {
        use crate::parsers::spotify::SpotifyParser;
        let lryics = api
            .get_lyrics(&best.id)
            .await?
            .ok_or("汽水音乐: 获取歌曲详情失败")?;
        if lryics.is_empty() {
            return Err("汽水音乐: 歌词内容为空".into());
        }
        Ok(SpotifyParser {}.parse(lryics)?)
    }
}
#[async_trait]
impl LyricsProvider for SodaMusicProvider {
    type Searcher = crate::searchers::soda_music::SodaMusicSearcher;
    type Api = crate::providers::soda_music::SodaMusicApi;
    type SearchResult = crate::searchers::soda_music::SodaMusicSearchResult;

    fn create_searcher(&self) -> Self::Searcher {
        crate::searchers::soda_music::SodaMusicSearcher::new()
    }
    fn create_api(&self) -> Self::Api {
        crate::providers::soda_music::SodaMusicApi::new()
    }
    fn label() -> &'static str {
        "汽水音乐"
    }

    async fn fetch_and_parse(
        api: &Self::Api,
        best: &Self::SearchResult,
    ) -> Result<Vec<LineInfo>, Box<dyn std::error::Error + Send + Sync>> {
        use crate::parsers::soda_music::SodaParser;
        use crate::parsers::IParsers;
        let detail = api
            .get_detail(&best.id)
            .await?
            .ok_or("汽水音乐: 获取歌曲详情失败")?;
        let lyric_info = detail.lyric.ok_or("汽水音乐: 歌曲没有歌词")?;
        let content = lyric_info.content.ok_or("汽水音乐: 无歌曲详细信息")?;
        if content.is_empty() {
            return Err("汽水音乐: 歌词内容为空".into());
        }
        Ok(SodaParser {}.parse(content)?)
    }
}

#[async_trait]
impl LyricsProvider for AppleMusicProvider {
    type Searcher = crate::searchers::applemusic::ApplemusicSearcher;
    type Api = crate::providers::applemusic::ApplemusicApi;
    type SearchResult = crate::searchers::applemusic::ApplemusicSearchResult;

    fn create_searcher(&self) -> Self::Searcher {
        crate::searchers::applemusic::ApplemusicSearcher::new(self.token.clone())
    }
    fn create_api(&self) -> Self::Api {
        crate::providers::applemusic::ApplemusicApi::new(self.token.clone())
    }
    fn label() -> &'static str {
        "applemusic"
    }

    async fn fetch_and_parse(
        api: &Self::Api,
        best: &Self::SearchResult,
    ) -> Result<Vec<LineInfo>, Box<dyn std::error::Error + Send + Sync>> {
        use crate::parsers::applemusic::AppleMusicParser;
        let detail = api
            .get_lyric(&best.id)
            .await?
            .ok_or("applemusic: 获取歌曲详情失败")?;
        let lyric_data = detail.data.ok_or("applemusic: 歌曲没有歌词")?;
        let u = lyric_data.get(0).unwrap();
        let att = u.attributes.as_ref().ok_or("applemusic: 无歌曲详细信息")?;
        let lyrics = att
            .ttml_localizations
            .as_ref()
            .ok_or("applemusic: 歌词内容为空")?;
        if lyrics.is_empty() {
            return Err("applemusic: 歌词内容为空".into());
        }
        Ok(AppleMusicParser {}.parse(lyrics.to_string())?)
    }
}
