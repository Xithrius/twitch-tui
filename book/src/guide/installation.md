# Installation

Once you have installed Rust, the following command can be used to build and install twitch-tui:

```sh
cargo install twitch-tui
```

This will automatically download mdBook from [crates.io], build it, and install it in Cargo's global binary directory (`~/.cargo/bin/` by default).

To uninstall, run the command `cargo uninstall twitch-tui`.

[rust installation page]: https://www.rust-lang.org/tools/install
[crates.io]: https://crates.io/

## Running twitch-tui

#### Config

Create the following config file `config.toml`. The full list of config is available in the [default-config.toml].

- Linux/MacOS: `~/.config/twt/config.toml`
- Windows: `%appdata%\twt\config.toml`

[default-config.toml]: https://github.com/Xithrius/twitch-tui/blob/main/default-config.toml

#### Auth

Get an OAuth token from [Twitch](https://twitchapps.com/tmi/), and place the value in the `token` variable in the `config.toml` that was previously generated.

#### Run it

Run the program with `twt` in the terminal to generate the default configuration at the paths below.

```sh
twt
```

#### Help and options

Run `twt --help` if you're looking for more options/arguments.

```sh
twt --help
```
