use async_trait::async_trait;
use crate::providers::soda_music::SodaMusicApi;
use super::{ISearcher, ISearchResult};
pub struct SodaMusicSearcher {
    api: SodaMusicApi,
}

impl SodaMusicSearcher {
    pub fn new() -> Self {
        Self { api: SodaMusicApi::new() }
    }
}

impl Default for SodaMusicSearcher {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ISearcher for SodaMusicSearcher {
    async fn search_for_results_by_string(&self, search_string: &str) -> Result<Vec<Box<dyn ISearchResult>>, Box<dyn std::error::Error + Send + Sync>> {
        let result = self.api.search(search_string).await?;
        let mut results: Vec<Box<dyn ISearchResult>> = Vec::new();

        let resp = result.ok_or_else(|| "SodaMusic: resp is None")?;
        let groups = resp.result_groups.ok_or_else(|| "SodaMusic: result_groups is None")?;

        for group in groups {
            let Some(items) = group.data else { continue };
            for item in items {
                let Some(entity) = item.entity else { continue };
                let Some(track) = entity.track else { continue };

                let title = track.name.unwrap_or_default();
                let artists: Vec<String> = track.artists
                    .unwrap_or_default()
                    .iter()
                    .filter_map(|a| a.name.clone())
                    .collect();
                let album = track.album.as_ref().and_then(|a| a.name.clone()).unwrap_or_default();
                let duration = track.duration.map(|d| d as u32);
                let id = track.id.unwrap_or_default();
                let trial = {
                    if let Some(preview) = track.preview {
                        if let Some(d) = preview.duration {
                            if let Some(s) = preview.start {
                                Some([s, d])
                            } else {
                                Some([0, d])
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                };
                results.push(Box::new(SodaMusicSearchResult {
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
        }

        Ok(results)
    }

    fn min_score(&self) -> i8 { 5 }
    fn get_split_char(&self) -> char {
        ','
    }
}

pub struct SodaMusicSearchResult {
    pub id: String,
    pub title: String,
    pub artists: Vec<String>,
    pub album: String,
    pub duration_ms: Option<u32>,
    pub match_score: i8,
    pub trial: Option<[u32; 2]>,
    pub is_trial: bool,
}

impl ISearchResult for SodaMusicSearchResult {
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

