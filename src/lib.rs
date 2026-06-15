use moka::sync::Cache;
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub struct TwitchEmoteParser {
    cache: Cache<String, Arc<TwitchEmote>>,
    channel_index: Mutex<HashMap<String, Vec<String>>>,
    token: String,
    client_id: String,
    client: Client,
}

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

#[derive(Clone, Debug)]
pub struct Emote {
    pub id: u32,
    pub start: u16,
    pub end: u16,
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

        Ok(Self {
            cache,
            channel_index: Mutex::new(HashMap::new()),
            token: token.to_string(),
            client_id: client_id.to_string(),
            client,
        })
    }

    pub fn get(&self, key: &str) -> Option<Arc<TwitchEmote>> {
        self.cache.get(key)
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

    pub async fn add_channel(&self, channel_id: &str) -> Result<(), reqwest::Error> {
        let response = self
            .client
            .get("https://api.twitch.tv/helix/chat/emotes")
            .query(&[("broadcaster_id", channel_id)])
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Client-ID", &self.client_id)
            .send()
            .await?
            .error_for_status()?;

        let parsed: EmoteResponse = response.json().await?;

        let mut names = Vec::new();

        for emote in parsed.data {
            let name = emote.name.clone();
            self.cache.insert(name.clone(), Arc::new(emote));
            names.push(name);
        }

        self.channel_index
            .lock()
            .unwrap()
            .insert(channel_id.to_string(), names);

        Ok(())
    }

    pub fn remove_channel(&self, channel_id: &str) {
        if let Some(names) = self.channel_index.lock().unwrap().remove(channel_id) {
            for name in names {
                self.cache.invalidate(&name);
            }
        }
    }
}
