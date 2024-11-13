# umu-wrapper

This is a simple wrapper for [umu-launcher](https://github.com/Open-Wine-Components/umu-launcher), which is then a wrapper for the Steam Runtime.

## Motivation

I always liked the simple approach of launching WINE applications with Bottles using the Bottles application library (Adding shortcuts to the Bottle, and then running them from the Bottles UI or with bottles-cli). However, It runs outside the control of Steam. So I created this.

Bottles runs the game in a separate environment, which means Steam Remote Play, Steam Input, Overlay and other features won't work. UMU-Launcher is designed to work with the Steam runtime, which means you can use all of these features directly.

However, it's also kind of cumbersome to use UMU-Launcher, since you have to either:

1. Run it from the terminal, and set all the environment variables for each game you want to run.
2. Do the above, but script it.
3. Create a profile for each game you want to run, as a separate TOML config.

umu-wrapper simply wraps around umu-launcher that automatically generates profiles from a master TOML config, allowing you to
re-use the same variables for multiple games. Think of it as just a more barebones version of Bottles, designed specifically to run with UMU-Launcher.

## Usage

1. Install umu-launcher
2. Create a TOML file with the following format:

```toml
# Optional: Global variables that will be applied to all games.
#[global]
# fallback to the default wine prefix if not set
# if this is still not set it will be generated from the game ID
#prefix = "/path/to/your/wineprefix"
# fallback to the default Proton runtime if not set
#proton = "/path/to/your/proton/runtime"

# A template is a template for a game profile, declaring reusable variables for multiple games.
[[template]]
name = "default"
# The Proton prefix for this template
prefix = "/path/to/your/wineprefix"
# The Proton runtime for this template
proton = "/path/to/your/proton/runtime"
store = "steam"

# Game profiles
[[profile]]
# An easy to remember ID for the game
name = "game1"
# The template to use for this game
template = "default"
# The game's UMU ID
id = "umu-1234567" # Steam app ID or any ID that UMU supports
# The game's executable
exe = "game1.exe"
# Optionally, arguments to pass to the game
args = ["-arg1", "-arg2"]
```


## todo

- [ ] MIME wrapper for .exe files, allowing you to launch executables directly from the file manager.
- [ ] Multiple configuration files, allowing templates to be shared.


