use serde::{Serialize, Deserialize};

use crate::sources::{ChannelPost, ChannelPostMedia};

pub struct TelegramClient {
    pub token: String,
}

pub enum TelegramResponse {
    Success,
    Error(u16, String),
}

#[derive(Serialize)]
struct TelegramInputMedia {
    r#type: String,
    media: String,
    caption: String,
}

#[derive(Deserialize)]
struct TelegramErrorResponse {
    description: String,
    error_code: u16,
}

impl TelegramClient {
    pub fn new(token: String) -> Self {
        Self { token }
    }

    pub fn create_post(&self, chat_id: String, post: ChannelPost) -> Result<TelegramResponse, reqwest::Error> {
        if post.media.len() == 0 {
            return Ok(TelegramResponse::Success);
            // TODO: send text
        }

        let mut media: Vec<TelegramInputMedia> = post.media.iter().map(|f| match f {
            ChannelPostMedia::Photo(url) => TelegramInputMedia {
                r#type: "photo".into(),
                media: url.into(),
                caption: String::with_capacity(0),
            },
            ChannelPostMedia::Video(url) => TelegramInputMedia {
                r#type: "video".into(),
                media: url.into(),
                caption: String::with_capacity(0),
            },
        }).collect();

        media[0].caption = format!("{}\n\nsrc: {}", post.text, post.source);

        let media = serde_json::to_string(&media).unwrap();

        let client = reqwest::blocking::Client::new();
        let res = client.post(format!("https://api.telegram.org/bot{}/sendMediaGroup", self.token))
            .query(&[("chat_id", chat_id), ("media", media)])
            .send()?;

        if !res.status().is_success() {
            let text = res.text().unwrap();
            let tgres = serde_json::from_str(&text)
                .unwrap_or(TelegramErrorResponse { error_code: 0, description: "(platfoxbot) Internal Error".into() });

            Ok(TelegramResponse::Error(tgres.error_code, tgres.description))
        } else {
            Ok(TelegramResponse::Success)
        }
    }
}

