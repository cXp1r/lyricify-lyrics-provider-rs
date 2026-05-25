use async_trait::async_trait;
use crate::providers::netease::NeteaseApi;
use super::{ISearcher, ISearchResult};
pub struct NeteaseSearcher {
    api: NeteaseApi,
}

impl NeteaseSearcher {
    pub fn new() -> Self {
        Self { api: NeteaseApi::new() }
    }
}

impl Default for NeteaseSearcher {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ISearcher for NeteaseSearcher {
    async fn search_for_results_by_string(&self, search_string: &str) -> Result<Vec<Box<dyn ISearchResult>>, Box<dyn std::error::Error + Send + Sync>> {
        let result = self.api.search(search_string, 1).await?;
        let mut results: Vec<Box<dyn ISearchResult>> = Vec::new();

        let data = result.result.ok_or_else(|| "Netease: result is None")?;
        let songs = data.songs.ok_or_else(|| "Netease: songs is None")?;

        for song in songs {
            let title = song.name.unwrap_or_default();
            let artists: Vec<String> = song.artists
                .unwrap_or_default()
                .iter()
                .filter_map(|a| a.name.clone())
                .collect();
            let album = song.album.as_ref().and_then(|a| a.name.clone()).unwrap_or_default();
            let duration = song.duration.map(|d| d as u32);
            let id = match &song.id {
                Some(serde_json::Value::Number(n)) => n.to_string(),
                Some(serde_json::Value::String(s)) => s.clone(),
                _ => String::new(),
            };
            let trial = {
                if let Some(trial) = self.api.get_detail(&id).await? {
                    if let Some(data) = trial.data {
                        if let Some(data) = data.get(0) {
                            if let Some(info) = &data.free_trial_info {
                                if let (Some(s), Some(e)) = (info.start, info.end) {
                                    Some([s as u32 * 1000, (e - s) as u32 * 1000])
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            };

            results.push(Box::new(NeteaseSearchResult {
                id,
                title,
                artists,
                album,
                duration_ms: duration,
                match_score: 0,
                trial,
                is_trial: false,
            }));
        }

        Ok(results)
    }

    fn get_split_char(&self) -> char {
        '/'
    }
}

#[derive(Debug, Clone)]
pub struct NeteaseSearchResult {
    pub id: String,
    pub title: String,
    pub artists: Vec<String>,
    pub album: String,
    pub duration_ms: Option<u32>,
    pub match_score: i8,
    pub trial: Option<[u32; 2]>,
    pub is_trial: bool,
}

impl ISearchResult for NeteaseSearchResult {
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


