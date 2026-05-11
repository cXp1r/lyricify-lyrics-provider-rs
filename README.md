# Lyrix
## 声明
- 虽然部分源码由ai移植[Lyricify-Lyrics-Helper](https://github.com/WXRIW/Lyricify-Lyrics-Helper)而来，但是每一行代码已经本人审计
- 本人新手，该项目同时用于熟悉rust基础语法

## 优点
- 封装了统一函数可以直接接收smtc信息进行歌词解析
- memchr予以的超高性能，无需预热或优化即可实现1ms一下解析

## 功能

- **Providers** — 网易云、QQ音乐、酷狗、汽水音乐的 API 客户端
- **Searchers** — 弱智评分机制 + 神人匹配字符串,返回最佳匹配
- **Parsers** — µs级别解析网易云,汽水,QQ音乐,酷狗音乐歌词,可解析**逐字高亮歌词**

## 安装

cargo add lyrix

或

在 `Cargo.toml` 中添加：
```toml
[dependencies]
lyrix = { version = "26.2.1" }
tokio = { version = "1", features = ["full"] }
```


## 快速上手

照着你的smtc获取到的传就是了,酷狗提供album_artist而不是album

指定参数
get_lyrics(
    title: &str,
    artist: Option<&str>,
    album: Option<&str>,
    album_artist: Option<&str>,
    duration_ms: u32,
)


### 访问解析/模型/工具模块

```rust
use lyrix::parsers;
use lyrix::models;
use lyrix::helpers;
```

## 支持的播放器

| 播放器 | 枚举值 | 进程名 | 歌词源 |
|--------|--------|--------|--------|
| 酷狗音乐 | `MusicPlayer::Kugou` | `KuGou.exe` | 酷狗 API |
| 网易云音乐 | `MusicPlayer::Netease` | `cloudmusic.exe` | 网易云 API（优先 YRC 逐字，回退 LRC） |
| QQ音乐 | `MusicPlayer::QQMusic` | `QQMusic.exe` | QQ音乐 API |
| 汽水音乐 | `MusicPlayer::SodaMusic` | `SodaMusic.exe` | 汽水音乐 API |

## 模块结构

```text
src/
├── lib.rs
├── smtc_lyrics.rs
├── models/
│   ├── mod.rs
│   ├── additional_file_info.rs
│   ├── file_info.rs
│   ├── line_info.rs
│   ├── lyrics_data.rs
│   ├── lyrics_types.rs
│   ├── sync_types.rs
│   └── track_metadata.rs
├── parsers/
│   ├── mod.rs
│   ├── attributes_helper.rs
│   ├── kugou.rs
│   ├── lrc.rs
│   ├── netease.rs
│   ├── qqmusic.rs
│   ├── soda_music.rs
│   └── decrypt/
│       ├── mod.rs
│       ├── krc.rs
│       └── qrc.rs
├── providers/
│   ├── mod.rs
│   ├── base_api.rs
│   ├── kugou.rs
│   ├── netease.rs
│   ├── proxy.rs
│   ├── qqmusic.rs
│   └── soda_music.rs
└── searchers/
    ├── mod.rs
    ├── kugou.rs
    ├── netease.rs
    ├── qqmusic.rs
    └── soda_music.rs
```

## 代理设置

```rust
use lyrix::providers::proxy;
use lyrix::providers::netease::NeteaseApi;

let client = proxy::create_proxy_client("127.0.0.1", 7890, None, None)?;
let api = NeteaseApi::with_client(client);
```

## 许可证

Apache-2.0
