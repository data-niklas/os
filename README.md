# Omnisearch (os)
A universal search engine. Image your browser URL bar search, but global and extendable.


## Status
Mostly complete and will be in low maintenance mode.

## Features
- Different UI backends: Wayland layer / CLI (gtk, ratatui, egui)
- Lots of different sources
- Fast (multi-core loading)

## Current sources
- Stdin: Dmenu support
- Applications: Launch linux applications using the XDG specification
- Systemctl: Shutdown / Reboot / Suspend your PC

- Cliphist: Display your recent clipboard entries using cliphist
- Linkding: List bookmarks from a linkding instance
- Duckduckgo: Search directly in Duckduckgo
- Zoxide: List zoxide directories
- Hstr: Display recent commands
- Eval: Adds math support and support for simple expressions
- SearchSites: Search in external websites. Define a prefix for each website.

## Roadmap
- [X] Default search source (dmenu like search)
- [X] Configuration from `.toml`
- [X] CLI arguments to enable / disable sources
- [X] Different result components:
    - [X] Text with (optional) icon
    - [X] Image
- [X] Result caching (e.g.) for web searches
- [X] History


## FAQ
- Q: Why the short name `os`, it sounds like operating system?
- A: `¯\_(ツ)_/¯`
