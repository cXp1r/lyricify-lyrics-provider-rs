use async_trait::async_trait;
use crate::providers::applemusic::ApplemusicApi;
use super::{ISearcher, ISearchResult, SearcherType};
pub struct ApplemusicSearcher {
    api: ApplemusicApi,
}

impl ApplemusicSearcher {
    pub fn new(token: String) -> Self {
        Self { api: ApplemusicApi::new(token) }
    }
}

impl Default for ApplemusicSearcher {
    fn default() -> Self {
        Self::new(String::new())
    }
}
//酷狗音乐SMTC只提供title artist albumArtist? 
//duration只能api拿了
#[async_trait]
impl ISearcher for ApplemusicSearcher {
    fn name(&self) -> &str { "Kugou" }
    fn display_name(&self) -> &str { "Kugou Music" }
    fn searcher_type(&self) -> SearcherType { SearcherType::Kugou }

    async fn search_for_results_by_string(&self, search_string: &str) -> Result<Vec<Box<dyn ISearchResult>>, Box<dyn std::error::Error + Send + Sync>> {
        let result = self.api.search(search_string).await?;
        let mut results: Vec<Box<dyn ISearchResult>> = Vec::new();

        if let Some(resp) = result {
            if let Some(res) = resp.results {
                if let Some(songs) = res.songs {
                    if let Some(songsv) = songs.data{
                        for song in songsv {
                            if let Some(info) = song.attributes{

                                let title = info.name.clone().unwrap_or_default();
                                let singer = info.artist_name.clone().unwrap_or_default();
                                let artists: Vec<String> = singer.split('、')//中文区顿号
                                    .map(|s| s.trim().to_string())
                                    .filter(|s| !s.is_empty())
                                    .collect();
                                let album = info.album_name.clone().unwrap_or_default();
                                let duration = info.duration_in_millis.map(|d| (d * 1000) as u32);
                                let has_lyrics = info.has_lyrics.clone().unwrap_or(false);
                                results.push(Box::new(ApplemusicSearchResult {
                                    title,
                                    artists,
                                    album,
                                    duration_ms: duration,
                                    has_lyrics,
                                    match_score: 0,
                                }));
                            }
                        }
                    }
                    
                }
            }
        }

        Ok(results)
    }
    fn min_score(&self) -> i8 { 5 }
    fn get_split_char(&self) -> char {
        '、'
    }
}

pub struct ApplemusicSearchResult {
    pub title: String,
    pub artists: Vec<String>,
    pub album: String,
    pub duration_ms: Option<u32>,  // snake_case
    pub has_lyrics: bool, 
    pub match_score: i8,
}

impl ISearchResult for ApplemusicSearchResult {
    fn title(&self) -> &str { &self.title }
    fn artists(&self) -> &[String] { &self.artists }
    fn album(&self) -> &str { &self.album }
    fn duration_ms(&self) -> Option<u32> { self.duration_ms }
    fn match_score(&self) -> i8 { self.match_score }
    fn set_match_score(&mut self, score: i8) { self.match_score = score; }
    fn as_any(&self) -> &dyn std::any::Any { self }
}
