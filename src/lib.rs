use moka::sync::Cache;
use reqwest::Client;
use serde::Deserialize;
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone, Debug, Deserialize)]
pub struct EmoteResponse {
    pub data: Vec<TwitchEmote>,
    pub template: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TwitchEmote {
    pub id: String,
    pub name: String,
    pub images: EmoteImages,
}

#[derive(Clone, Debug, Deserialize)]
pub struct EmoteImages {
    pub url_1x: String,
    pub url_2x: String,
    pub url_4x: String,
}

pub struct TwitchEmoteParser {
    cache: Cache<String, Arc<TwitchEmote>>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Emote {
    id: u32,
    start: u16,
    end: u16,
}

impl TwitchEmoteParser {
    pub async fn new(token: &str, client_id: &str) -> Result<Self, reqwest::Error> {
        let client = Client::new();

        let response = client
            .get("https://api.twitch.tv/helix/chat/emotes/global")
            .header("Authorization", format!("Bearer {}", token))
            .header("Client-ID", client_id)
            .send()
            .await?
            .error_for_status()?;

        let parsed: EmoteResponse = response.json().await?;

        let cache = Cache::builder()
            .max_capacity(10_000)
            .time_to_live(Duration::from_secs(60 * 60 * 12))
            .build();

        for emote in parsed.data {
            cache.insert(emote.name.clone(), Arc::new(emote));
        }

        Ok(Self { cache })
    }

    pub fn get(&self, key: &str) -> Option<Arc<TwitchEmote>> {
        self.cache.get(&key.to_owned())
    }

    pub fn populate_string(&self, input: &str) -> Vec<Emote> {
        let mut result = Vec::new();

        let mut start = 0;

        for word in input.split_whitespace() {
            if let Some(pos) = input[start..].find(word) {
                let real_start = start + pos;
                let real_end = real_start + word.len();

                if let Some(emote) = self.cache.get(word) {
                    let id = emote.id.parse::<u32>().unwrap_or(0);

                    result.push(Emote {
                        id,
                        start: real_start as u16,
                        end: (real_end - 1) as u16,
                    });
                }

                start = real_end;
            }
        }

        result
    }
}
