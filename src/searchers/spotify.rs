use crate::providers::spotify::SpotifyApi;
use async_trait::async_trait;
use super::{ISearcher, ISearchResult, SearcherType};

pub struct SpotifySearcher {
    api: SpotifyApi,
}

impl SpotifySearcher {
    pub fn new(cookie: String) -> Self {
        Self { api: SpotifyApi::new(cookie) }
    }
}

impl Default for SpotifySearcher {
    fn default() -> Self {
        Self::new("".to_string())
    }
}

#[async_trait]
impl ISearcher for SpotifySearcher {
    fn name(&self) -> &str { "Spotify" }
    fn display_name(&self) -> &str { "Spotify" }
    fn searcher_type(&self) -> SearcherType { SearcherType::Spotify }

    async fn search_for_results_by_string(&self, search_string: &str) -> Result<Vec<Box<dyn ISearchResult>>, Box<dyn std::error::Error + Send + Sync>> {
        let result = self.api.search(search_string).await?;
        
        let mut results: Vec<Box<dyn ISearchResult>> = Vec::new();
        if let Some(resp) = result{
            if let Some(data) = resp.data {
                if let Some(search_v2) = data.search_v2 {
                    
                        if let Some(items) = search_v2.items {
                            for song in items {
                                if let Some(w) = song.item {
                                    if let Some(t) = w.data {
                                        let id = t.id.unwrap();
                                        let title = t.name.unwrap();
                                        let album = t.album_of_track.unwrap();
                                        let album_name = album.name.unwrap();
                                        let artist = t.artists.unwrap();
                                        let artists: Vec<String> = artist.items.unwrap()
                                            .iter()
                                            .filter_map(|s| s.profile.as_ref().unwrap().name.clone())
                                            .collect();
                                        let duration_ms = t.duration.unwrap().total_milliseconds;
                                        results.push(Box::new(SpotifySearchResult{
                                            id,
                                            title,
                                            artists,
                                            album: album_name,
                                            duration_ms,
                                            trial: None,
                                            is_trial: false,
                                            match_score: 0,
                                        }));
                                        
                                    }
                                }
                            }
                        }
                        return Err("QQMusicApi: No song".into());
                    
                }
                return Err("QQMusicApi: No data".into());
            }
            return Err("QQMusicApi: No req_1".into());      
        }
        return Err("QQMusicApi: No resp".into());
    }
    fn get_split_char(&self) -> char {
        '/'
    }
}


#[derive(Debug, Clone)]
pub struct SpotifySearchResult {
    pub id: String,
    pub title: String,
    pub artists: Vec<String>,
    pub album: String,
    pub duration_ms: Option<u32>,
    pub match_score: i8,
    pub trial: Option<[u32; 2]>,
    pub is_trial: bool,
}
impl ISearchResult for SpotifySearchResult {
    fn title(&self) -> &str { &self.title }
    fn artists(&self) -> &[String] { &self.artists }
    fn album(&self) -> &str { &self.album }
    fn duration_ms(&self) -> Option<u32> { self.duration_ms }
    fn match_score(&self) -> i8 { self.match_score }
    fn set_match_score(&mut self, score: i8) { self.match_score = score; }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn trial(&self) -> Option<[u32; 2]> { self.trial }
    fn set_trial(&mut self, i: bool) { self.is_trial = i; }
}
