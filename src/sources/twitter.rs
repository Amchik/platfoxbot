use std::collections::HashMap;

use serde::Deserialize;

use super::{ChannelPost, ChannelPostMedia};

pub struct TwitterClient {
    /// Twitter bearer token
    pub token: String,
    /// user_id -> tweet_id
    pub last_ids: HashMap<String, u64>,
}

#[derive(Clone, Debug)]
pub struct TwitterTweet {
    pub id: u64,
    pub text: String,
    pub media: Vec<TwitterMedia>,
}
#[derive(Clone, Debug)]
pub enum TwitterMedia {
    Photo(String),
    Video(String),
//    Gif(String),  TODO:
}

#[derive(Deserialize)]
struct TwitterUserTimeline {
    data: Vec<TwitterRawTweet>,
    includes: TwitterTimelineIncludes,
}
#[derive(Deserialize)]
struct TwitterRawTweet {
    id: String,
    text: String,
    attachments: Option<TwitterRawTweetAttachments>,
}
#[derive(Deserialize)]
struct TwitterRawTweetAttachments {
    media_keys: Vec<String>,
}
#[derive(Deserialize)]
struct TwitterTimelineIncludes {
    media: Vec<TwitterTimelineMedia>,
}
#[derive(Deserialize)]
struct TwitterTimelineMedia {
    media_key: String,
    r#type: String,
    url: Option<String>,
    variants: Option<Vec<TwitterTimelineMediaVariants>>,
}
#[derive(Deserialize)]
struct TwitterTimelineMediaVariants {
    bitrate: Option<u32>,
    url: String,
}

impl TwitterClient {
    pub fn new(token: String) -> Self {
        Self { token, last_ids: HashMap::new() }
    }

    pub fn fetch_from(&mut self, user_id: String) -> Result<Vec<TwitterTweet>, reqwest::Error> {
        let client = reqwest::blocking::Client::new();
        let res = client
            .get(format!("https://api.twitter.com/2/users/{}/tweets", user_id))
            .query(&[
                ("exclude",      "replies,retweets"),
                ("tweet.fields", "attachments"),
                ("expansions",   "attachments.media_keys"),
                ("media.fields", "type,url,variants"),
                ("max_results",  "5"),
            ])
            .header("Authorization", format!("Bearer {}", self.token))
            //.header("User-Agent", "platfoxbot/2.0.0 (+https://ceheki.org)")
            .send()?;

        let data: TwitterUserTimeline = serde_json::from_str(&res.text()?).unwrap();

        let last_id = *self.last_ids.get(&user_id).unwrap_or(&0);

        let mut res = vec![];

        for tweet in data.data {
            let id = tweet.id.parse().unwrap();

            if id <= last_id {
                continue;
            }

            let media = if let Some(attachments) = tweet.attachments {
                attachments.media_keys.iter()
                    .map(|f| data.includes.media.iter().position(|r| &r.media_key == f))
                    .map(|idx| &data.includes.media[idx.unwrap()])
                    .map(|m| if m.r#type == "photo" {
                        TwitterMedia::Photo(m.url.clone().unwrap())
                    } else {
                        TwitterMedia::Video(m.variants.as_ref().unwrap().iter().max_by_key(|v| v.bitrate).unwrap().url.clone())
                    })
                    .collect()
            } else {
                vec![]
            };

            res.push(TwitterTweet {
                id,
                text: tweet.text,
                media,
            });
        }

        // Update last id
        if let Some(fart) = res.first() {
            self.last_ids.insert(user_id, fart.id);
        }

        Ok(res)
    }
}

impl Into<ChannelPost> for TwitterTweet {
    fn into(self) -> ChannelPost {
        ChannelPost {
            text: self.text,
            media: self.media.iter()
                .map(|f| match f {
                    TwitterMedia::Photo(s) => ChannelPostMedia::Photo(s.clone()),
                    TwitterMedia::Video(s) => ChannelPostMedia::Video(s.clone()),
                })
                .collect(),
            source: "twitter".into()
        }
    }
}

