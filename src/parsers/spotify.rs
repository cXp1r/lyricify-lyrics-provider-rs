//先用serde凑合着,太慢了就手搓一个memchr
use serde::Deserialize;
use serde_json::from_str;
use crate::models::{LineInfo};

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LyricResult {
    pub sync_type: Option<String>,
    pub lines: Option<Vec<Line>>,
}
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Line {
    pub start_time_ms: Option<u32>,
    pub words: Option<String>,
    //目前没遇到过,不知道具体格式pub syllables: Vec<>,
    pub end_time_ms: Option<u32>,
}

pub struct SpotifyParser {
    
}

impl SpotifyParser {
    pub fn parse(&self, lyrics: String) -> Result<Vec<LineInfo>, String> {
        let mut lineinfo = Vec::new();
        let result: LyricResult =
            from_str(&lyrics).map_err(|e| e.to_string())?;
        if let Some(sync_type) = result.sync_type{
            match sync_type.as_str() {
                "LINE_SYNCED" => {
                    if let Some(lines) = result.lines{
                        for line in lines {
                            let Some(st) = line.start_time_ms else {
                                return Err("SpotifyParser: Failed to find start_time".into());
                            };
                            let Some(et) = line.end_time_ms else {
                                return Err("SpotifyParser: Failed to find end_time".into());
                            };
                            let et = if et > st { (et - st) as u16} else { et as u16 };
                            let Some(words) = line.words else {
                                return Err("SpotifyParser: Failed to find words".into());
                            };
                            lineinfo.push(
                                LineInfo { start_time: st, duration: et, text: words, syllables: vec![] }
                            );
                        }
                    }
                }
                _ => {
                    return Err("SpotifyParser: unknown sync_type".into());
                }
            }
        }
        Ok(lineinfo)
    }
}