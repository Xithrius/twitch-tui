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

## Emotes

Use a terminal that fully supports the graphics protocol, such as [kitty](https://sw.kovidgoyal.net/kitty/) or [WezTerm](https://wezfurlong.org/wezterm/).

Enable the emotes by setting `twitch_emotes`, `betterttv_emotes` and/or `seventv_emotes` to `true`.

The emotes will be downloaded to `~/.cache/twt/` on Linux/MacOs and `%appdata%\twt\cache\` on Windows.

## Run it

Run `twt` in the terminal. For help, `twt --help`.
