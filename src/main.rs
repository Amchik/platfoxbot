use std::{collections::HashMap, fs, process::exit};

use clap::Parser;
use futures::future::join_all;

use crate::{
    sources::twitter::TwitterClient,
    telegram::{TelegramClient, TelegramResponse},
};

mod config;
mod sources;
mod telegram;

#[derive(Parser)]
#[clap(version, about)]
struct Args {
    /// Toml configuration. See README for more info.
    #[clap(short, long, value_parser, default_value = "platfoxbot.toml")]
    config: String,

    /// Cache file. Can be redefined in config.
    #[clap(
        short = 's',
        long,
        value_parser,
        default_value = "platfoxbot.cache.json"
    )]
    cache: String,

    /// Ignores config cache file and use by command line arguments.
    #[clap(long, value_parser, default_value_t = false)]
    ignore_config_cache_file: bool,

    /// Create cache file and exit
    #[clap(long, value_parser)]
    create_cache: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    println!("Try reading config from {}...", args.config);

    let bytes = match fs::read(args.config) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Failed to open file for reading: {}", e);
            exit(1);
        }
    };

    let cfg = String::from_utf8_lossy(&bytes);
    let cfg: config::Config = toml::from_str(&cfg).unwrap();

    let tw = TwitterClient::new(cfg.twitter.token);
    let tg = TelegramClient::new(cfg.telegram.token);

    let cache_filename = if args.ignore_config_cache_file {
        args.cache
    } else {
        cfg.cache.unwrap_or(args.cache)
    };

    let mut cache: HashMap<String, u64> = match fs::read(&cache_filename) {
        Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or_default(),
        _ => HashMap::new(),
    };

    // Maybe it bad idea
    let tweets = {
        let jobs = cfg.twitter.ids.into_iter().map(|user_id| {
            let last_id = cache.get(&user_id).copied();

            tw.fetch_from(user_id, last_id)
        });
        let tweets = join_all(jobs).await;
        // NOTE: 2 for loops may be also bad idea, use .map(|v| { ...; v }) aka lazy-loading
        for tweet in tweets.iter().flatten().filter(|v| !v.is_empty()) {
            let tweet = &tweet[0];

            cache.insert(tweet.author_id.to_string(), tweet.id);
        }

        tweets
            .into_iter()
            .flatten()
            .flat_map(|v| v.into_iter().rev())
    };

    // Write cache
    let ids_export = serde_json::to_string(&cache).unwrap();
    fs::write(cache_filename, ids_export).expect("Caching IDs");

    for tweet in tweets {
        let tweet_id = tweet.id;

        let res = tg
            .create_post(&cfg.telegram.chat_id, tweet.into())
            .await
            .unwrap_or_else(|e| TelegramResponse::Error(0, e.to_string()));

        if let TelegramResponse::Error(code, description) = res {
            eprintln!("Failed to repost {}: {description} (code {code})", tweet_id);
        }
    }
}
