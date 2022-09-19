# platfoxbot v2

(Platinum Foxes Bot) twitter to telegram image reposter.

## Usage

`$ platfoxbot --help`

## Configuration

Default config file is `platfoxbot.toml`. It can be changed via `-c <config>`

```toml
# Example config

# (optional) Cache file path.
# Default is platfoxbot.cache.json
cache = "platfoxbot.cache.json"

[telegram]
# Telegram token from @BotFather
token = ""
# Telegram chat id. (note: use BOT ID, not ids that appears in web{k,z})
# Also, its a string
chat_id = "-1001522104971"

[twitter]
# Twitter bearer token
token = ""
# Array (of strings) of ids to scan. 
ids = [ "1170746320115646464" ]

```

