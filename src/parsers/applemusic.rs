use crate::models::LineInfo;
use memchr::{memchr, memmem};

pub struct AppleMusicParser {

}

impl AppleMusicParser {
    #[allow(unused_variables)]
    pub fn parse_time(&self, tag: &str, syllables_by_word: bool) -> Result<u32,String> {
        //完全要求格式化
        //00:03:26.910
        //012345678901
        //时:分:秒.毫秒
        let hours = tag[0..2].parse::<u32>()
            .map_err(|_e| "Applemusic Parser: failed to parse hours")?;
        let minutes = tag[3..5].parse::<u32>()
            .map_err(|_e| "Applemusic Parser: failed to parse hours")?;
        let seconds = tag[6..8].parse::<u32>()
            .map_err(|_e| "Applemusic Parser: failed to parse hours")?;
        let centis = tag[9..11].parse::<u32>()
            .map_err(|_e| "Applemusic Parser: failed to parse hours")?;

        Ok(hours * 3_600_000 +minutes * 60_000 + seconds * 1_000 + centis * 10)
    }
    pub fn parse(&self, lyrics: String) -> Result<Vec<LineInfo>, String> {
        let mut lineinfo: Vec<LineInfo> = Vec::new();
        
        let Some(mut cpos) = memmem::find(lyrics.as_bytes(), b"div") else {
            return Err("Applemusic Parser: lyrics body not found".into());
        };
        let ulyrics = &lyrics[cpos..];
        let len = ulyrics.len();
        let bytes = ulyrics.as_bytes();
        let mut it = memmem::find_iter(bytes, "=\"");
        while cpos < len {
            
            //<p begin=\"00:03:26.910\" end=\"00:03:30.830\">すれ違っても ずっと君でいて</p><p begin=\"00:03:30.830\" end=\"00:03:47.830\">きっと会いに行くから</p>
            let st = match it.next() {
                Some(u) => {//定位到时长开始
                    cpos = u + 2;
                    //println!("{}",cpos);
                    let Some(c) = memchr(b'\"', &bytes[cpos..]) else {
                        return Err("Applemusic Parser: start_time not found".into());
                    };
                    //println!("{}",&ulyrics[cpos..cpos + c]);
                    self.parse_time(&ulyrics[cpos..cpos + c], false)?
                },
                None => {
                    break;
                }
            };
            let et = match it.next() {
                Some(u) => {//定位到时长开始
                    cpos = u + 2;
                    let Some(c) = memchr(b'\"', &bytes[cpos..]) else {
                        return Err("Applemusic Parser: start_time not found".into());
                    };
                    self.parse_time(&ulyrics[cpos..cpos + c], false)?
                },
                None => {
                    break;
                }
            };
            cpos += 2;
            let Some(s) = memchr(b'>',&bytes[cpos..]) else {
                return Err("Applemusic Parser: failed to parse lyrics".into());
            };
            cpos += s + 1;
            let Some(s) = memchr(b'<',&bytes[cpos..]) else {
                return Err("Applemusic Parser: failed to parse lyrics".into());
            };

            lineinfo.push(LineInfo {
                    start_time: st,
                    duration: (et - st) as u16,
                    text: ulyrics[cpos..cpos + s].to_string(),
                    syllables: vec![],
            });
        }
        Ok(lineinfo)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let lyrics = "<tt xmlns=\"http://www.w3.org/ns/ttml\" xmlns:itunes=\"http://music.apple.com/lyric-ttml-internal\" itunes:timing=\"Line\" xml:lang=\"en\"><head><metadata><iTunesMetadata xmlns=\"http://music.apple.com/lyric-ttml-internal\"><translations/><songwriters><songwriter>Kanae Nakamura (pka Kanata Nakamura)</songwriter><songwriter>Yuki Honda</songwriter></songwriters></iTunesMetadata></metadata></head><body dur=\"00:03:47.830\"><div begin=\"00:00:04.540\" end=\"00:03:47.830\"><p begin=\"00:00:04.540\" end=\"00:00:11.100\">知りたくて知りたくない このままでいい</p><p begin=\"00:00:11.100\" end=\"00:00:34.070\">(マルバツ)がつくのなら ずっとずっと明日にならないで</p><p begin=\"00:00:34.070\" end=\"00:00:41.360\">遠いセカイのことだと思っていた (思っていた)</p><p begin=\"00:00:41.360\" end=\"00:00:48.030\">わたしには関係ないこんなキモチ (こんなキモチ)</p><p begin=\"00:00:48.030\" end=\"00:00:55.400\">君にだったらアリかもね ココロの中を見せても だってきっと変わらない</p><p begin=\"00:00:55.400\" end=\"00:00:59.250\">ココロの位置がわかったよ なんだか苦しくなるよ</p><p begin=\"00:00:59.250\" end=\"00:01:02.820\">ふいに変わる 風向きが</p><p begin=\"00:01:02.820\" end=\"00:01:06.680\">明日は何になる? やがて君になる</p><p begin=\"00:01:06.680\" end=\"00:01:10.320\">繊細な中身 覗いてみて</p><p begin=\"00:01:10.320\" end=\"00:01:14.260\">モヤモヤしてる 気持ちがバレたら</p><p begin=\"00:01:14.260\" end=\"00:01:32.980\">君は逃げてしまうかな</p><p begin=\"00:01:32.980\" end=\"00:01:40.100\">なんとなく毎日が輝いてる (輝いてる)</p><p begin=\"00:01:40.100\" end=\"00:01:47.280\">特別を特別と気付かないまま (気付かないまま)</p><p begin=\"00:01:47.280\" end=\"00:01:54.800\">そんなことより明日は 2人でどこかへ行こう 今の距離は壊さずに</p><p begin=\"00:01:54.800\" end=\"00:01:58.630\">少しずつ壊れていく 2人の距離はそのうち</p><p begin=\"00:01:58.630\" end=\"00:02:01.510\">限界越えて ああ ゼロに</p><p begin=\"00:02:01.510\" end=\"00:02:05.520\">明日は誰になる? やがて君になる</p><p begin=\"00:02:05.520\" end=\"00:02:09.380\">どんなに早く逃げたとして</p><p begin=\"00:02:09.380\" end=\"00:02:13.010\">すれ違っても ずっと君でいて</p><p begin=\"00:02:13.010\" end=\"00:02:32.350\">きっと会いに行くから</p><p begin=\"00:02:32.350\" end=\"00:02:47.250\">近くて (まだ) 遠くて (ああ) もう少しで届くのに</p><p begin=\"00:02:47.250\" end=\"00:02:53.250\">透明なガラスの向こう</p><p begin=\"00:02:53.250\" end=\"00:02:57.550\">君の剥き出しのココロ</p><p begin=\"00:02:57.550\" end=\"00:03:04.290\">守ってあげたくて 触れられず</p><p begin=\"00:03:04.290\" end=\"00:03:08.520\">明日は何になる? やがて君になる</p><p begin=\"00:03:08.520\" end=\"00:03:12.220\">繊細な中身 覗いてみて</p><p begin=\"00:03:12.220\" end=\"00:03:16.030\">モヤモヤしてる 気持ちがバレたら</p><p begin=\"00:03:16.030\" end=\"00:03:19.680\">君は逃げてしまうかな</p><p begin=\"00:03:19.680\" end=\"00:03:23.340\">明日は誰になる? やがて君になる</p><p begin=\"00:03:23.340\" end=\"00:03:26.910\">どんなに早く逃げたとして</p><p begin=\"00:03:26.910\" end=\"00:03:30.830\">すれ違っても ずっと君でいて</p><p begin=\"00:03:30.830\" end=\"00:03:47.830\">きっと会いに行くから</p></div></body></tt>";
        
        let parser = AppleMusicParser{};
        match parser.parse(lyrics.to_string()) {
            Ok(m) => println!("{:?}",m),
            Err(e) => println!("{:?}",e),
        }
        return ;
        
    }
}