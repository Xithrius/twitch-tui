# Usage as a flake

[![FlakeHub](https://img.shields.io/endpoint?url=https://flakehub.com/f/Xithrius/twitch-tui/badge)](https://flakehub.com/flake/Xithrius/twitch-tui)

Add twitch-tui to your `flake.nix`:

```nix
{
  inputs.twitch-tui.url = "https://flakehub.com/f/Xithrius/twitch-tui/*.tar.gz";

  outputs = { self, twitch-tui }: {
    # Use in your outputs
  };
}

```
