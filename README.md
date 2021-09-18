# Twitch Chat IRC, in the terminal.

### What it looks like:

![image](https://user-images.githubusercontent.com/15021300/133889088-7ec17848-b6c2-4e80-8dea-47f4b5b9553a.png)

### Setup:

1. Copy everything from `default-config.toml` to a new file called `config.toml` to set everything you need.
2. Create a `.env` file and set `TOKEN` to your oauth token from [here](https://twitchapps.com/tmi/) (
   ex. `TOKEN=oauth:asdfasdfasdf`).
3. run `cargo run --release`, and you're good to go!
