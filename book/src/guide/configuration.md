# Configuration

## Config file

After running `twt` for the first time, a config file will be generated at the following locations, depending on your OS:

- Linux/MacOS: `~/.config/twt/config.toml`
- Windows: `%appdata%\twt\config.toml`

You can find the default configuration values [here](https://github.com/Xithrius/twitch-tui/blob/main/default-config.toml).

## Authentication

You can get a Twitch token to use for authentication from one of two places:

- https://twitchapps.com/tmi/: Generates a token with minimal scopes
- https://twitchtokengenerator.com/: Generates a token with custom scopes

The latter is needed for emotes and what users you follow. If you want those features, enable the `user:read:follows` and `user:read:emotes` scopes.

The minimal scope generator will only give you scopes of `chat:read`, `chat:edit`, `whispers:read`, `whispers:edit`, and `channel:moderate`. Be aware that whispers are currently not supported.

Once you have your token, put `:oauth` at the start if it's not there already, then place it in one of two places:

1. The `token` variable in the `config.toml` that was previously generated.
2. The environment variable `TWT_TOKEN`.

The environment variable will be used first, even if a token exists in `config.toml`. If one doesn't exist there, your config token will be used.

## Emotes

Currently, only the [graphics protocol for kitty]() is supported, so any other terminal without it won't be able to render emotes.

Enable the emotes by setting `twitch_emotes`, `betterttv_emotes` and/or `seventv_emotes` to `true`.

The emotes will be downloaded to `~/.cache/twt/` on Linux/MacOs and `%appdata%\twt\cache\` on Windows.

## Run it

Run `twt` in the terminal. For help, `twt --help`.
