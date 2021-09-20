# Twitch Chat IRC, in the terminal.

### What it looks like:

![image](https://user-images.githubusercontent.com/15021300/133889088-7ec17848-b6c2-4e80-8dea-47f4b5b9553a.png)

### Keybinds:
- `?` for the keybinds help window.
- `i` to insert text. Exit this mode with `Esc`.
- `Esc` exists out of layered windows, such as going from insert mode, then normal, to exiting the application.
- `c` to go from whatever window (such as the help window) to chat.
- `q` to quit out of the entire application, given you're not in insert mode.

### Setup:

1. Make sure you have both Cargo and installed from the [rust-lang website](https://www.rust-lang.org/learn/get-started). Make sure the Cargo binary folder is appended to your `$PATH` environment variable.
2. Get an OAuth token from [Twitch](https://twitchapps.com/tmi/), and have it ready to put into the `token` variable in the `config.toml` file that you create.
3. Run `cargo install twitch-terminal-chat` and follow the instructions that it prints.
5. You should now be able to run `ttc` from anywhere now. Have fun!
