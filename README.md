# RUSTUBE

Rust youtube mp3 dowloader.

## Prerequisites

* `youtube-dl`
* `ffmpeg`

## Usage

1. Copy youtube link to `link.txt` line by line.
2. `cargo run --release`
3. See `mp3_dir`. That's it!

## Caution

* Do not use link in your playlists. If then, you can download just the first song in your playlist. 