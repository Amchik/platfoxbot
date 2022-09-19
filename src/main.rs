use std::{fs, process::exit};

use clap::Parser;

use crate::{sources::twitter::TwitterClient, telegram::{TelegramClient, TelegramResponse}};

mod sources;
mod telegram;
mod config;

#[derive(Parser)]
#[clap(version, about)]
struct Args {
    /// Toml configuration. See README for more info.
    #[clap(short, long, value_parser, default_value = "platfoxbot.toml")]
    config: String,

    /// Cache file. Can be redefined in config.
    #[clap(short = 's', long, value_parser, default_value = "platfoxbot.cache.json")]
    cache: String,

    /// Ignores config cache file and use by command line arguments.
    #[clap(long, value_parser, default_value_t = false)]
    ignore_config_cache_file: bool
}

fn main() {
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

    let mut tw = TwitterClient::new(cfg.twitter.token);
    let tg = TelegramClient::new(cfg.telegram.token);

    let cache_filename = if args.ignore_config_cache_file {
        args.cache
    } else {
        cfg.cache.unwrap_or(args.cache)
    };

    if let Ok(bytes) = fs::read(&cache_filename) {
        tw.last_ids = serde_json::from_slice(&bytes).unwrap_or_default();
    }

    let mut failed = false;

    for user_id in cfg.twitter.ids {
        let posts = tw.fetch_from(user_id).unwrap();

        for post in posts.iter().rev() {
            let res = tg.create_post(cfg.telegram.chat_id.clone(), post.clone().into()).unwrap();

            if let TelegramResponse::Error(code, description) = res {
                eprintln!("[FAILTURE] Failed to post to telegram. {code}: {description}");
                eprintln!("{:#?}", post);
                failed = true;
            }
        }
    }

    let ids_export = serde_json::to_string(&tw.last_ids).unwrap();

    fs::write(cache_filename, ids_export).expect("Caching IDs");

    if failed {
        exit(2);
    }
}
