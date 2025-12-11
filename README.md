# Pwwwd

Phillips's wgpu-based Wayland wallpaper daemon.

## Overview

pwwwd is a wallpaper daemon for Wayland compositors, implemented in Rust and using wgpu for rendering. The goal is to provide a simple, GPU-accelerated mechanism to set and animate desktop wallpapers on Wayland, handling resizing, transitions, and daemon-based wallpaper switching via a command-line interface.

## Features

- :heavy_check_mark: Render wallpapers to desktop
- :heavy_check_mark: Basic resize options: `no`, `crop`, `fit`, `stretch`
- :heavy_check_mark: Daemon control for wallpaper switching (via command line)
- :heavy_check_mark: Wallpaper transition animations
    - :heavy_check_mark: Variaties of transition animation types
        - :heavy_check_mark: Xfd
        - :heavy_check_mark: Wipe
        - :construction: More transition types ...
    - :heavy_check_mark: Easing transition animation, with easing function options
        - :heavy_check_mark: Support widely used easing functions ...
        - :heavy_check_mark: ... or customize your easing function with cubic-bezier curve
- :construction: Restore last used wallpaper on startup
    - :heavy_check_mark: Load last wallpaper
    - :x: Display last wallpaper with the same options
- :x: Multiple monitor support with individual wallpapers
- :x: Video and animated image support

## Dependencies

- A wayland compositor that supports `wlr-layer-shell` protocol.
- (If you want build pwwwd from source) a valid `rust` installation. MSRV = 1.88

## Usage

Launch pwwwd daemon to display walllpaper on your desktop ...

```bash
pwwwd load <img-path>
```

... or if you want to restore last used wallpaper.

```bash
pwwwd restore
```

You can switch wallpaper at runtime, with CLI controller `pwww`.

```bash
pwww img <img-path>
```

For more information, run `help` subcommand.

```bash
pwwwd help
pwww help
```

## Build from source

```bash
cargo build
```
