# Installation

## Cargo

After [installing Rust](https://www.rust-lang.org/tools/install), use the following command to install twitch-tui:

```sh
cargo install twitch-tui
```

For a specific version, head over to the [releases page](https://github.com/Xithrius/twitch-tui/releases) and select the release you want.

Installing a version such as `2.0.0-alpha.1` would be `cargo install twitch-tui --version "2.0.0-alpha.1"`.

(PPT and AUR repos coming soon)

To uninstall, run the command `cargo uninstall twitch-tui`.

## Nix

twitch-tui is also a [Nix Flake](https://nixos.wiki/wiki/Flakes)! You can build and run it on nix using:

```sh
nix run github:Xithrius/twitch-tui
```
