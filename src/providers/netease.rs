use super::base_api::BaseApi;
use serde::Deserialize;
use std::collections::HashMap;

pub struct NeteaseApi {
    api: BaseApi,
}

impl NeteaseApi {
    fn netease_headers() -> HashMap<String, String> {
        let mut h = HashMap::new();
        h.insert("cookie".to_string(), super::base_api::COOKIE.to_string());
        h
    }

    pub fn new() -> Self {
        Self {
            api: BaseApi::new(Some("https://music.163.com/"), Some(Self::netease_headers())),
        }
    }

    pub fn with_client(client: reqwest::Client) -> Self {
        Self {
            api: BaseApi::with_client(client, Some("https://music.163.com/"), Some(Self::netease_headers())),
        }
    }

    /// 搜索歌曲
    pub async fn search(&self, keyword: &str, search_type: i32) -> Result<SearchResult, reqwest::Error> {
        let mut params = HashMap::new();
        params.insert("s".to_string(), keyword.to_string());
        params.insert("type".to_string(), search_type.to_string());
        params.insert("limit".to_string(), "20".to_string());
        params.insert("offset".to_string(), "0".to_string());

        let resp = self.api.post_form_async(
            "https://music.163.com/api/search/get/web",
            &params,
        ).await?;


        let parsed: SearchResult = serde_json::from_str(&resp).unwrap_or(SearchResult { code: -1, result: None });


        Ok(parsed)
    }

    /// 获取歌词
    pub async fn get_lyric(&self, id: &str) -> Result<LyricResult, reqwest::Error> {
        let mut params = HashMap::new();
        params.insert("id".to_string(), id.to_string());
        params.insert("lv".to_string(), "-1".to_string());
        params.insert("kv".to_string(), "-1".to_string());
        params.insert("tv".to_string(), "-1".to_string());
        params.insert("rv".to_string(), "-1".to_string());
        params.insert("yv".to_string(), "-1".to_string());
        params.insert("ytv".to_string(), "-1".to_string());
        params.insert("yrv".to_string(), "-1".to_string());


        let resp = self.api.post_form_async(
            "https://interface3.music.163.com/api/song/lyric/v1",
            &params,
        ).await?;

        let parsed: LyricResult = serde_json::from_str(&resp).unwrap_or(LyricResult::default());


        Ok(parsed)
    }

    /// 获取歌曲详情
    pub async fn get_detail(&self, id: &str) -> Result<Option<DetailResult>, reqwest::Error> {
        let url = "/api/song/enhance/player/url/v1";
        let body = format!(
            r#"{{"ids":"[\"{id}\"]","level":"exhigh","encodeType":"aac","csrf_token":""}}"#
        );
        let p = crate::parsers::decrypt::netease::eapi_encrypt(url, &body).unwrap_or(String::new());
        
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0")
            .build()?;

        let res = client
            .post(format!("https://music.163.com/eapi/song/enhance/player/url/v1"))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Cookie", "WEVNSM=1.0.0; os=pc; osver=Microsoft-Windows-11-Professional-build-114514-64bit; channel=netease; mode=System Product Name;appver=3.1.32.205206")
            .form(&[("params", p.as_str())])
            .send()
            .await?;
        let resp = res.text().await?;
        Ok(serde_json::from_str(&resp).ok())
    }
}

impl Default for NeteaseApi {
    fn default() -> Self {
         Self::new()
    }
}
// ===== Response Models =====

#[derive(Debug, Deserialize, Default)]
pub struct SearchResult {
    pub code: i64,
    pub result: Option<SearchResultData>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SearchResultData {
    pub songs: Option<Vec<Song>>,
    pub song_count: Option<i64>,
    pub albums: Option<Vec<Album>>,
    pub album_count: Option<i64>,
}

#[derive(Debug, Deserialize, Default)]
pub struct LyricResult {
    pub code: Option<i64>,
    pub nolyric: Option<bool>,
    pub uncollected: Option<bool>,
    pub lrc: Option<Lyrics>,
    pub klyric: Option<Lyrics>,
    pub tlyric: Option<Lyrics>,
    pub romalrc: Option<Lyrics>,
    pub yrc: Option<Lyrics>,
    pub ytlrc: Option<Lyrics>,
    pub yromalrc: Option<Lyrics>,
}

#[derive(Debug, Deserialize, Default)]
pub struct Lyrics {
    pub version: Option<i64>,
    pub lyric: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DetailResult {
    pub data: Option<Vec<Detail>>,
    pub code: Option<i64>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Detail {
    pub free_trial_info: Option<Trial>,
}

#[derive(Debug, Deserialize, Default)]
pub struct Trial  {
    pub start: Option<u8>,
    pub end: Option<u8>,
}


#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Song {
    pub name: Option<String>,
    pub id: Option<serde_json::Value>,
    #[serde(alias = "ar")]
    pub artists: Option<Vec<Ar>>,
    #[serde(alias = "al")]
    pub album: Option<Al>,
    #[serde(alias = "dt")]
    pub duration: Option<i64>,
    pub publish_time: Option<i64>,
}

#[derive(Debug, Deserialize, Default)]
pub struct Ar {
    pub id: Option<i64>,
    pub name: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct Al {
    pub id: Option<i64>,
    pub name: Option<String>,
    #[serde(rename = "picUrl")]
    pub pic_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Album {
    pub name: Option<String>,
    pub id: Option<i64>,
    pub size: Option<i64>,
    pub artist: Option<Artist>,
}

#[derive(Debug, Deserialize)]
pub struct Artist {
    pub name: Option<String>,
    pub id: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test() {
        let n = NeteaseApi::new();
        let _x = n.get_detail("1939760528").await;
    }
}