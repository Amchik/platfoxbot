use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub cache: Option<String>,

    pub telegram: TelegramConfig,
    pub twitter: TwitterConfig,
}

#[derive(Serialize, Deserialize)]
pub struct TelegramConfig {
    pub token: String,
    pub chat_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct TwitterConfig {
    pub token: String,
    pub ids: Vec<String>,
}
