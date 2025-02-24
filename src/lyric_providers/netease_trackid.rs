use std::time::Duration;
use isahc::HttpClient;
use isahc::prelude::Configurable;
use mpris::Metadata;
use crate::lyric_parser::{
    LyricLine,
    parse_netease_lyrics,
};

#[derive(Clone)]
pub struct NeteaseTrackIDLyricProvider {}


impl NeteaseTrackIDLyricProvider {
    pub async fn get_lyric_by_metadata(
        &self,
        metadata: &Metadata,
        config: crate::config::SharedConfig,
    ) -> (Vec<LyricLine>, bool, bool) {
        let client = HttpClient::builder()
            .timeout(Duration::from_secs(
                config.read().unwrap().online_search_timeout
            ))
            .cookies()
            .build()
            .expect("初始化网络请求失败!");
        let ncm_api = ncm_api::MusicApi::from_client(client);
        if let Some(track_id) = metadata.track_id() {
            // let music_id = track_id.rsplit("/").next().unwrap().parse::<u64>().unwrap();
            if let Ok(music_id) = track_id.as_str().rsplit("/").next().unwrap().parse::<u64>() {
                let mut success = !config.read().unwrap().online_search_retry;
                let mut try_count = 0;
                let max_retries = config.read().unwrap().max_retries;

                #[allow(unused_assignments)]
                while !success && try_count < max_retries {
                    println!("Trying to get lyric for track_id: {}", music_id);
                    let lyric_result = ncm_api.song_lyric(music_id).await;
                    if let Ok(lyric_result) = lyric_result {
                        success = true;
                        let lyric_lines = lyric_result.lyric;
                        let tlyric_lines = lyric_result.tlyric;
                        return (
                            parse_netease_lyrics(lyric_lines, tlyric_lines),
                            true, false
                        );
                    } else {
                        try_count += 1;
                    }
                }
                return (Vec::new(), false, true);  // 达到最大重试次数
            }
        }
        (Vec::new(), false, true)
    }

    pub fn is_available_by_metadata(&self, metadata: &Metadata) -> bool {
        metadata.track_id().is_some() && metadata.track_id().unwrap().as_str().rsplit("/").next().unwrap().parse::<u64>().is_ok()
    }
}
