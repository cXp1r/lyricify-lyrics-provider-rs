use async_trait::async_trait;
use crate::providers::applemusic::ApplemusicApi;
use super::{ISearcher, ISearchResult, SearcherType};
use crate::models::ITrackMetadata;

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
                            let id = song.id.clone().unwrap_or_default();
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
                                    id,
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
    async fn make_search_string(&self, track: &dyn ITrackMetadata) -> Option<String> {
        let combined = format!(
            "{}",
            track.title().unwrap_or_default(),
        ).replace(" - ", " ").trim().to_string();

        if combined.is_empty() {
            None
        } else {
            Some(combined)
        }
    }
    fn compare_track(&self, track: &dyn ITrackMetadata, result: &dyn ISearchResult) -> i8 {
        let mut score = 0i8;

        // 第一步没必要覆写,强制留着了
        let track_title = track.title().unwrap_or_default().to_lowercase();
        let result_title = result.title().to_lowercase();
        if !track_title.is_empty() && !result_title.is_empty() {
            if track_title == result_title {
                score += 4;
            } else if result_title.contains(&track_title) || track_title.contains(&result_title) {
                score += 2;
            } else {
                let clean_track = self.clean_title(&track_title);
                let clean_result = self.clean_title(&result_title);
                if clean_track == clean_result {
                    score += 3;
                } else if clean_result.contains(&clean_track) || clean_track.contains(&clean_result) {
                    score += 1;
                }
            }
        }
        //println!("{}:{}",result_title,score);

        // Artist match
        let d: Vec<String> = track
            .artist()
            .unwrap_or_default()   //防止下面崩溃
            .split("—")
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .collect();
        let artists: Vec<String> = d.get(0).unwrap_or(&String::new())
            .split("、")
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .collect();
        for a in &artists {
            if result.artists().iter().any(|b| {
                let b = b.to_lowercase();
                a == &b || a.contains(&b) || b.contains(a)
            }) {
                score += 1;
            }
        }

        //println!("{} {}",result.artists().join("||"),score);
        // Album match
        let track_album = d.get(1).unwrap_or(&String::new()).clone();
        let result_album = result.album().to_lowercase();
        if !track_album.is_empty() && !result_album.is_empty() && track_album == result_album {
            score += 1;
        }

        //println!("{} {}",result_album,score);
        // Album artist match
        let track_album_artist = self.clean_title(&track.album_artist().unwrap_or_default().to_lowercase());
        let result_album_artist = result.album_artists().unwrap_or_default().to_vec();

        if result_album_artist.iter().any(|s:&String| s.contains(&track_album_artist)) {
            score += 1;
        }
        //println!("(kugou) score:{}",score);
        if let Some(duration_ms) = track.duration_ms() {
            if let Some(result_duration_ms) = result.duration_ms() {
                let diff = duration_ms as i64 - result_duration_ms as i64;
                if diff == 0 { // 完全匹配
                    
                    score += 2;
                }else if diff <= 1000 { // 1秒内认为时长匹配
                    score += 1;
                }
                
            }
        }
        //println!("{} {}\n",result.duration_ms().unwrap_or_default(),score);
        score
    }
}

pub struct ApplemusicSearchResult {
    pub id: String,
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
