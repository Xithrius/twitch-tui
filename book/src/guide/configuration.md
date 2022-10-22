# Configuration

## Config file

After running `twt` for the first time, a config file will be generated at the following locations, depending on your OS:

- Linux/MacOS: `~/.config/twt/config.toml`
- Windows: `%appdata%\twt\config.toml`

You can find the default configuration values [here](https://github.com/Xithrius/twitch-tui/blob/main/default-config.toml).

## Auth

Get an OAuth token from [Twitch](https://twitchapps.com/tmi/), and put the value in one of two places:

1. The `token` variable in the `config.toml` that was previously generated.
2. The environment variable `TWT_TOKEN`.

The environment variable will be used first, even if a token exists in `config.toml`. If one doesn't exist there, your config token will be used.

## Run it

Run `twt` in the terminal. For help, `twt --help`.
