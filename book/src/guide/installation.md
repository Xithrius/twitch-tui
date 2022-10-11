# Installation

After [installing Rust](https://www.rust-lang.org/tools/install), use the following command to install twitch-tui:

```sh
cargo install twitch-tui
```

For a specific version, head over to the [releases page](https://github.com/Xithrius/twitch-tui/releases) and select the release you want.

Installing a version such as `2.0.0-alpha.1` would be `cargo install twitch-tui --version "2.0.0-alpha.1"`.

(PPT and AUR repos coming soon)

To uninstall, run the command `cargo uninstall twitch-tui`.

## Configuration

### Config file

After running `twt` for the first time, a config file will be generated at the following locations, depending on your OS:

- Linux/MacOS: `~/.config/twt/config.toml`
- Windows: `%appdata%\twt\config.toml`

You can find the default configuration values [here](https://github.com/Xithrius/twitch-tui/blob/main/default-config.toml).

### Auth

Get an OAuth token from [Twitch](https://twitchapps.com/tmi/), and place the value in the `token` variable in the `config.toml` that was previously generated.

### Run it

Run `twt` in the terminal. For help, `twt --help`.
