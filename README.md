# asteroids

Simple [Asteroids](https://en.wikipedia.org/wiki/Asteroids_(video_game)) game clone in Rust language, using [macroquad](https://github.com/not-fl3/macroquad) engine.

Based on an [Asteroids tutorial for Lua and LÖVE 11](https://simplegametutorials.github.io/love/asteroids/).

## Playing the game from source

### Dependencies

The main dependency — the rust compiler. To get it, follow [rustup.rs](https://rustup.rs/) instructions.

#### Web, Windows, macOS

No other external dependencies are required.

#### Linux

Followed libs may be required:

```bash
# ubuntu system dependencies
apt install pkg-config libx11-dev libxi-dev libgl1-mesa-dev libasound2-dev

# fedora system dependencies
dnf install libX11-devel libXi-devel mesa-libGL-devel alsa-lib-devel

# arch linux system dependencies
 pacman -S pkg-config libx11 libxi mesa-libgl alsa-lib
```

### Running the game

```bash
cargo run
```

## License

It is in the **public domain** under the [WTFPL](http://www.wtfpl.net/about/) license.
