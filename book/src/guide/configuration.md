# Configuration

## Config file

After running `twt` for the first time, a config file will be generated at the following locations, depending on your OS:

- Linux/MacOS: `~/.config/twt/config.toml`
- Windows: `%appdata%\twt\config.toml`

You can find the default configuration values [here](https://github.com/Xithrius/twitch-tui/blob/main/default-config.toml).

## Authentication

The most convenient way to get a Twitch token is to use twitchtokengenerator.com. [Here is a quick link with the required scopes already enabled](https://twitchtokengenerator.com/?scope=channel:moderate+chat:edit+chat:read+moderator:manage:chat_messages+user:read:chat+user:read:emotes+user:read:follows+user:write:chat+moderator:manage:banned_users&auth=auth_stay). Once generated copy the "ACCESS TOKEN".

The above token has the following scopes enabled:

```
channel:moderate
chat:edit
chat:read
moderator:manage:chat_messages
user:read:chat
user:read:emotes
user:read:follows
user:write:chat
moderator:manage:banned_users
```

Once you have a token, put `oauth:` at the start if it's not there already, then place it in one of two places:

1. The `token` variable in the `config.toml` that was previously generated.
2. The environment variable `TWT_TOKEN`.

The environment variable will be used first, even if a token exists in `config.toml`. If one doesn't exist there, your config token will be used.

## Emotes

Currently, only the [graphics protocol for kitty]() is supported, so any other terminal without it won't be able to render emotes.

Enable the emotes by setting `twitch_emotes`, `betterttv_emotes` and/or `seventv_emotes` to `true`.

The emotes will be downloaded to `~/.cache/twt/` on Linux/MacOs and `%appdata%\twt\cache\` on Windows.

## Run it

Run `twt` in the terminal. For help, `twt --help`.
