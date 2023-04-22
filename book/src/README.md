# Introduction

`twitch-tui` is a Terminal User Interface for reading and interacting with Twitch chat users.

The ideal setup would be this application and [streamlink](https://github.com/streamlink/streamlink), to obliterate all need for a browser while watching streams.

This book is still under construction, so some sections might still be missing. Feel free to [submit an issue](https://github.com/Xithrius/twitch-tui/issues/new/choose) if you find that something needs improvement.

## Quick start

This application has vim and emacs inspired keybinds. That means you will start out in visual mode.

The default start screen is the dashboard. This can be configured with `start_state` in the config file.

To go to a channel, either hit `Enter` or an integer between square brackets to go to the corresponding channel.

To enter insert mode after getting into a channel, press `i`. To drop back to visual mode, press `esc`.

You can go back to the dashboard by hitting `S` (`Shift + s`).

Insert modes at this time are message search (`Ctrl + f`), channel swapper (`s`), and the previously mentioned insert mode (`i`).

To see all the options with keybinds, either head over to the [keybinds section](https://xithrius.github.io/twitch-tui/keybinds/index.html) of this book, or press `?` while in visual mode.
