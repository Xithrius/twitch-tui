# Twitch Chat IRC, in the terminal.

### What it looks like:

![image](https://user-images.githubusercontent.com/15021300/133889088-7ec17848-b6c2-4e80-8dea-47f4b5b9553a.png)

### Keybinds:
- `?` for the keybinds window.
- `i` to insert text. Exit this mode with `Esc`.
- `Esc` exists out of layered windows, such as going from insert mode to normal, to exiting the application.
- `c` to go from whatever window (such as the help window) to chat.
- `q` to quit out of the entire application, given you're not in insert mode.

### Setup:

1. Copy everything from `default-config.toml` to a new file called `config.toml` to set everything you need.
2. Create a `.env` file and set `TOKEN` to your oauth token from [here](https://twitchapps.com/tmi/) (
   ex. `TOKEN=oauth:asdfasdfasdf`).
3. run `cargo run --release`, and you're good to go!
