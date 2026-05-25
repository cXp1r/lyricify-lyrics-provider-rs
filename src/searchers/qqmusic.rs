use async_trait::async_trait;
use crate::providers::qqmusic::QQMusicApi;
use super::{ISearcher, ISearchResult};

pub struct QQMusicSearcher {
    api: QQMusicApi,
}

impl QQMusicSearcher {
    pub fn new() -> Self {
        Self { api: QQMusicApi::new() }
    }
}

impl Default for QQMusicSearcher {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ISearcher for QQMusicSearcher {
    async fn search_for_results_by_string(&self, search_string: &str) -> Result<Vec<Box<dyn ISearchResult>>, Box<dyn std::error::Error + Send + Sync>> {
        let result = self.api.search(search_string).await?;
        let mut results: Vec<Box<dyn ISearchResult>> = Vec::new();

        let resp = result.ok_or_else(|| "QQMusic: resp is None")?;
        let req1 = resp.req_1.ok_or_else(|| "QQMusic: req_1 is None")?;
        let data = req1.data.ok_or_else(|| "QQMusic: data is None")?;
        let body = data.body.ok_or_else(|| "QQMusic: body is None")?;
        let song_list = body.song.ok_or_else(|| "QQMusic: song is None")?;
        let songs = song_list.list.ok_or_else(|| "QQMusic: list is None")?;

        for song in songs {
            let title = song.title.or(song.name).unwrap_or_default();
            let artists: Vec<String> = song.singer
                .unwrap_or_default()
                .iter()
                .filter_map(|s| s.title.clone())
                .collect();
            let album = song.album.as_ref().and_then(|a| a.title.clone()).unwrap_or_default();
            let duration = song.interval.map(|i| (i * 1000) as u32);
            let mid = song.mid.unwrap_or_default();
            let id  = song.id.unwrap_or_default();
            let trial = if let Some(file) = song.file {
                if let (Some(b), Some(e)) = (file.b_30s, file.e_30s) {
                    Some([b, e - b])
                } else {
                    None
                }
            } else {
                None
            };
            results.push(Box::new(QQMusicSearchResult {
                id,
                mid,
                title,
                artists,
                album,
                duration_ms: duration,
                match_score: 0,
                trial,
                is_trial: false,
            }));
        }

        if results.is_empty() {
            return Err("QQMusic: No songs".into());
        }
        Ok(results)
    }
    fn get_split_char(&self) -> char {
        '/'
    }
}

pub struct QQMusicSearchResult {
    pub mid: String,
    pub id: u32,
    pub title: String,
    pub artists: Vec<String>,
    pub album: String,
    pub duration_ms: Option<u32>,
    pub match_score: i8,
    pub trial: Option<[u32; 2]>,
    pub is_trial: bool,
}

impl ISearchResult for QQMusicSearchResult {
    fn title(&self) -> &str { &self.title }
    fn artists(&self) -> &[String] { &self.artists }
    fn album(&self) -> &str { &self.album }
    fn duration_ms(&self) -> Option<u32> { self.duration_ms }
    fn match_score(&self) -> i8 { self.match_score }
    fn set_match_score(&mut self, score: i8) { self.match_score = score; }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn trial(&self) -> Option<[u32; 2]> { self.trial }
    fn set_trial(&mut self, i: bool) { self.is_trial = i; }
    fn is_trial(&self) -> bool { self.is_trial }
}

