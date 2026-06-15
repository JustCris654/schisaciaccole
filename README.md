# Schisaciaccole

A countdown timer for **time-boxed Scrum ceremonies**.

Scrum ceremonies are time-boxed: regardless of when the team actually gets into
the meeting room, the meeting should end at its designated time. Schisaciaccole
makes that easy — instead of asking "how long do we have left?", you pick the
**wall-clock time the meeting must end** and the timer counts down precisely to
that moment.

## How it works

When you launch the app it shows the **next 8 quarter-hour marks** (00, 15, 30,
45 past the hour) starting from the upcoming quarter. That covers meetings of up
to **2 hours**.

- The options refresh automatically every 3 seconds, so they are always accurate
  even if the app sits open for a while before the meeting starts.
- Pick the slot when the ceremony has to end.
- The timer counts down to exactly that time and plays a sound when it reaches
  zero.
- Press **Stop** to end early and go back to the selection screen.

Example: it's 09:07 and the daily stand-up must wrap by 09:15. Open the app, tap
`09:15`, and the timer ends precisely at 09:15 — not 15 minutes from when you
pressed the button.

## Screenshots

| Selection | Timer |
|-----------|-------|
| ![Selection page](assets/screenshot_selection_page.png) | ![Timer page](assets/screenshot_timer_page.png) |

## Requirements

- [Rust](https://www.rust-lang.org/tools/install) (stable toolchain, `cargo`)

The default font (**Plus Jakarta Sans**) is bundled — no extra setup needed.

## Font setup (optional)

The app ships with **Plus Jakarta Sans** bundled, so it works out of the box.

**Chillax** looks nicer and is the preferred font, but it is **not** bundled —
to use it, download it and swap two pairs of commented lines in
[`ui/app-window.slint`](ui/app-window.slint).

1. Download Chillax: <https://api.fontshare.com/v2/fonts/download/chillax>
2. Unzip it.
3. Place the OTF Medium weight at exactly this path:

   ```
   ui/assets/fonts/Chillax_Variable/Fonts/OTF/Chillax-Medium.otf
   ```

4. In `ui/app-window.slint`, swap the comments on the import lines (top of file):

   ```slint
   import "../assets/fonts/Chillax_Variable/Fonts/OTF/Chillax-Medium.otf";
   // import "../assets/fonts/Plus_Jakarta_Sans/static/PlusJakartaSans-Medium.ttf";
   ```

5. ...and on the `default-font-family` lines inside `MainWindow`:

   ```slint
   // default-font-family: "Plus Jakarta Sans";
   default-font-family: "Chillax";
   ```

## Build & run

```sh
cargo run
```

For a release build:

```sh
cargo run --release
```

## Packaging

The app icon lives at `assets/icon.png` and the `game_over.mp3` sound ships as a
resource. At runtime the sound is located relative to the executable, so it
works from every package layout (dev, deb, macOS app, Arch pkg).

### Linux (.deb) and macOS (.app) — cargo-bundle

```sh
cargo install cargo-bundle          # once

cargo bundle --release --format deb   # Linux  -> target/release/bundle/deb/*.deb
cargo bundle --release --format osx   # macOS  -> *.app (must run on macOS)
```

Bundle metadata (name, identifier, icon, bundled resources) lives in the
`[package.metadata.bundle]` section of `Cargo.toml`.

### Arch Linux (pacman)

pacman can't install a `.deb`. Use the bundled PKGBUILD instead.

> **Run `makepkg` from inside `packaging/arch/`, never from the repo root.**
> makepkg uses `./src` and `./pkg` as its work dirs — at the repo root `./src`
> is the crate's own source directory and makepkg would wipe it. The PKGBUILD
> builds a clean copy under its own `src/`, so the crate is never touched.

```sh
cd packaging/arch
makepkg -f                  # rustup users without the pacman 'cargo' pkg: makepkg -fd
sudo pacman -U schisaciaccole-*.pkg.tar.zst
```

This installs the binary, the sound, the icon, a `.desktop` launcher and the
license under the standard `/usr` paths.

## Logging

The app uses `env_logger`. Control verbosity with the `RUST_LOG` environment
variable (default is `warn`):

```sh
RUST_LOG=info cargo run     # callback events: selection, start/pause, stop, sound
RUST_LOG=debug cargo run    # also the periodic option refresh
```
