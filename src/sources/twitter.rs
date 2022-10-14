use std::collections::HashMap;

use serde::Deserialize;

use super::{ChannelPost, ChannelPostMedia};

pub struct TwitterClient {
    /// Twitter bearer token
    pub token: String,
    /// HashMap of known since_ids in format user_id -> last_id
    pub last_ids: HashMap<String, u64>,
}

#[derive(Clone, Debug)]
pub struct TwitterTweet {
    pub id: u64,
    pub author_id: u64,
    pub author_name: String,
    pub text: String,
    pub media: Vec<TwitterMedia>,
}
#[derive(Clone, Debug)]
pub enum TwitterMedia {
    Photo(String),
    Video(String),
}

#[derive(Deserialize)]
struct TwitterUserTimeline {
    #[serde(default)]
    data: Vec<TwitterRawTweet>,
    #[serde(default)]
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
    #[serde(default)]
    media_keys: Vec<String>,
}
#[derive(Deserialize, Default)]
struct TwitterTimelineIncludes {
    #[serde(default)]
    media: Vec<TwitterTimelineMedia>,
    #[serde(default)]
    users: Vec<TwitterTimelineUser>,
}
#[derive(Deserialize)]
struct TwitterTimelineUser {
    id: String,
    name: String,
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
        Self {
            token,
            last_ids: HashMap::new(),
        }
    }

    pub fn fetch_from(&mut self, user_id: String) -> Result<Vec<TwitterTweet>, reqwest::Error> {
        let last_id = self.last_ids.get(&user_id);

        let query = {
            let mut query = HashMap::from([
                ("exclude", "replies,retweets".into()),
                ("tweet.fields", "attachments,author_id".into()),
                ("expansions", "attachments.media_keys,author_id".into()),
                ("media.fields", "type,url,variants".into()),
                ("max_results", "5".into()),
            ]);

            if let Some(last_id) = last_id {
                query.insert("since_id", last_id.to_string());
            }

            query
        };

        let client = reqwest::blocking::Client::new();
        let res = client
            .get(format!(
                "https://api.twitter.com/2/users/{}/tweets",
                user_id
            ))
            .query(&query)
            .header("Authorization", format!("Bearer {}", self.token))
            .send()?;

        let text = res.text()?;
        let data: TwitterUserTimeline =
            serde_json::from_str(&text).expect("JSON schema is invalid");

        let last_id = last_id.copied().unwrap_or_default();

        let mut res = vec![];
        let author = data
            .includes
            .users
            .iter()
            .find(|f| f.id == user_id)
            .expect("api doesn't returned author object");

        for tweet in data.data {
            let id = tweet.id.parse().expect("tweet id is not a number");

            if id <= last_id {
                continue;
            }

            let media = tweet
                .attachments
                .iter()
                .flat_map(|attachments| &attachments.media_keys)
                .map(|f| data.includes.media.iter().position(|r| &r.media_key == f))
                .map(|idx| &data.includes.media[idx.expect("api returned invalid media_key")])
                .map(|m| {
                    if m.r#type == "photo" {
                        TwitterMedia::Photo(m.url.clone().unwrap())
                    } else {
                        TwitterMedia::Video(
                            m.variants
                                .as_ref()
                                .unwrap()
                                .iter()
                                .max_by_key(|v| v.bitrate)
                                .expect("api doesn't returned normal video")
                                .url
                                .clone(),
                        )
                    }
                })
                .collect();

            res.push(TwitterTweet {
                id,
                author_id: user_id.parse().unwrap(),
                author_name: author.name.clone(),
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

impl From<TwitterTweet> for ChannelPost {
    fn from(tweet: TwitterTweet) -> ChannelPost {
        ChannelPost {
            text: tweet.text,
            media: tweet
                .media
                .iter()
                .map(|f| match f {
                    TwitterMedia::Photo(s) => ChannelPostMedia::Photo(s.clone()),
                    TwitterMedia::Video(s) => ChannelPostMedia::Video(s.clone()),
                })
                .collect(),
            source: format!("twitter // {}", tweet.author_name),
            source_url: Some(format!(
                "https://twitter.com/{}/status/{}",
                tweet.author_name, tweet.id
            )),
        }
    }
}
