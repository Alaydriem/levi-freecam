<div align="center">

# Alaydriem's Freecam

**A smooth free camera with keyframed camera paths for Minecraft Bedrock on Windows, loaded by LeviLamina.**

[![Release](https://img.shields.io/github/v/release/alaydriem/levi-freecam?include_prereleases)](https://github.com/alaydriem/levi-freecam/releases)
[![Build](https://img.shields.io/github/actions/workflow/status/alaydriem/levi-freecam/build.yml?branch=master&label=build)](https://github.com/alaydriem/levi-freecam/actions/workflows/build.yml)
[![License](https://img.shields.io/github/license/alaydriem/levi-freecam)](./LICENSE)
![Minecraft Bedrock](https://img.shields.io/badge/Minecraft_Bedrock-1.26.51-62B47A)
[![LeviLamina](https://img.shields.io/badge/LeviLamina-26.51.6-4A90D9)](https://github.com/LiteLDev/LeviLamina)
![Windows](https://img.shields.io/badge/Windows-x64-0078D6)

[Install](#-installation) · [Controls](#%EF%B8%8F-controls) · [Configuration](#%EF%B8%8F-configuration) · [FAQ](#-faq) · [Build](#%EF%B8%8F-building-from-source)

</div>

Alaydriem's Freecam lets the camera leave your player and fly anywhere. Your player stays frozen where you left it. Set points along a route and create simple camera paths for in-game realtime recording via OBS

> [!NOTE]
> This is a **client-side** mod. It sends nothing to the server. To the server, your player stands still.

---

## ✨ Features

- **🎥 Free camera**
  - Fly with the movement keys and look with the mouse
  - Change the speed with the mouse wheel.
  - Smooth motion can be turned off for direct, instant control.
- **🧍 Frozen player**
  - While the camera is free, your player does not walk, turn, jump, use or attack.
- **🖥️ Clean screen**
  - The HUD, the hand and the crosshair are hidden while freecam is on. Press <kbd>F1</kbd> to show the HUD again.
- **🛤️ Camera paths**
  - Add points along a route and play the path back with easing. Each point is a camera box drawn in the world, joined by a dashed line with an arrow on each part. The selected point's box is green and the others are blue.
  - When you add a point while the crosshair is on a block or an entity, the point looks at that target. Playback keeps the camera aimed at it, so orbit shots stay on the subject.
- **🎞️ Playback bar**
  - A small bar with rewind, play, a scrubber and the point tools. Press <kbd>Ctrl</kbd> + <kbd>E</kbd> to show or hide it.
  - With the bar open you can still fly with the keys, look around with the right mouse button held, and change the speed with the mouse wheel.
- **📸 Screenshots**
  - Save a PNG of the freecam view from the bar. The bar and the path markers are hidden for the shot.
- **🛡️ Safe exits**
  - Freecam turns off by itself when you take damage, die, change dimension or leave the world, or when the server takes control of the camera.

---

## 📦 Installation

You need [LeviLamina](https://github.com/LiteLDev/LeviLamina) for the **client**, on a supported game version (see [Compatibility](#-compatibility)).

Install with [lip](https://github.com/LiteLDev/lip). Run this command in your game folder:

```powershell
lip install github.com/alaydriem/levi-freecam#client
```

To remove the mod:

```powershell
lip uninstall github.com/alaydriem/levi-freecam
```

<details>
<summary><b>Manual install</b></summary>

1. Download `alaydriem-freecam-<version>-client-windows-x64.zip` from the [latest release](https://github.com/alaydriem/levi-freecam/releases/latest).
2. Extract it into your game's `mods` folder. The result must be `mods/alaydriem-freecam/alaydriem-freecam.dll`.
3. Start the game.

The folder name must stay `alaydriem-freecam`. LeviLamina does not load the mod from a folder with a different name.

</details>

---

## ✅ Compatibility

| Mod version | Minecraft Bedrock (Windows) | LeviLamina |
| --- | --- | --- |
| 0.1.0 | 1.26.51 | 26.51.6 or later 26.51.x |

The mod hooks into the game directly. A new game version needs a new mod release.

---

## ⌨️ Controls

| Key | Action | When |
| --- | --- | --- |
| <kbd>Ctrl</kbd> + <kbd>P</kbd> | Freecam on / off | In a world |
| <kbd>W</kbd> <kbd>A</kbd> <kbd>S</kbd> <kbd>D</kbd> | Move | Freecam on |
| <kbd>Space</kbd> / <kbd>Shift</kbd> | Move up / down | Freecam on |
| Hold <kbd>Ctrl</kbd> | Move 3 times faster | Freecam on |
| Mouse | Look | Freecam on, bar hidden |
| Mouse wheel | Change the speed. With the bar open, point away from the bar | Freecam on |
| <kbd>Ctrl</kbd> + <kbd>[</kbd> | Smooth motion on / off | Freecam on |
| <kbd>Ctrl</kbd> + <kbd>E</kbd> | Show or hide the playback bar. While a path plays: stop playback | Freecam on |
| <kbd>Ctrl</kbd> + <kbd>B</kbd> | Add a path point at the playhead | Freecam on, path not playing |
| <kbd>Del</kbd> | Delete the selected point. With no selected point, it does nothing | Freecam on, path not playing |
| Hold right mouse button and move | Look around. Press away from the bar; the cursor hides and comes back where it was when you release | Bar open |
| <kbd>F1</kbd> | Show or hide the HUD (the game's own key; it also shows name tags and the numbers of mods such as Light Levels) | Freecam on |

Key combinations work only when no game menu or chat is open.

While the bar is open, the mouse cursor shows and moving the mouse does not turn the camera. To look around, hold the right mouse button away from the bar and move the mouse. You can still move with the keys, and the mouse wheel still changes the speed when the cursor is away from the bar.

When a path plays, the bar and the markers hide. Playback stops at the end of the path, or when you press <kbd>Ctrl</kbd> + <kbd>E</kbd>. The camera then stays where it stopped.

### Playback bar

| Button | What it does |
| --- | --- |
| Rewind | Go to the start of the path. The camera moves there |
| Play | Play the path from the playhead. At the end of the path, play starts from the start |
| Timecode | The playhead time and the full path time |
| Speed | The playback speed, in blocks per second. It is the flying speed, so change it with the mouse wheel |
| Scrubber | Drag to move along the path. The camera follows. Each point is a mark |
| Select | Click a mark to select a point. The camera and the playhead move to it. Click it again to clear the selection |
| Re-place | Move the selected point to the camera (shown when a point is selected) |
| Delete | Delete the selected point (shown when a point is selected) |
| Add point | Add a point after the last point at or before the playhead |
| Clear points (✕) | Delete all points. Click two times within 3 seconds |
| Screenshot | Save a PNG to `%USERPROFILE%\Pictures\Screenshots` (the folder is a setting). A message on the bar says if it was saved or failed |

> [!TIP]
> Every key is a setting in `config.json`, written as a [Windows virtual-key code](https://learn.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes). <kbd>Ctrl</kbd> is the `modifier` key for every combination, and it is also the fast key. <kbd>Del</kbd> is pressed on its own.

---

## ⚙️ Configuration

The config file is `mods/alaydriem-freecam/config/config.json`. A key that you remove from the file uses its default. A value out of range is changed to the nearest allowed value. If the file cannot be read, the mod uses the defaults.

<details>
<summary><b>Full reference</b></summary>

Speeds are in blocks per second. Keys are Windows virtual-key codes.

| Section | Setting | Default | What it does |
| --- | --- | --- | --- |
| `keys` | `toggle` | `0x50` (<kbd>P</kbd>) | With the modifier: freecam on / off |
| | `smoothToggle` | `0xDB` (<kbd>[</kbd>) | With the modifier: smooth motion on / off |
| | `addPoint` | `0x42` (<kbd>B</kbd>) | With the modifier: add a path point |
| | `cursor` | `0x45` (<kbd>E</kbd>) | With the modifier: show or hide the bar. <kbd>Tab</kbd> is not allowed (the game uses it) |
| | `delete` | `0x2E` (<kbd>Del</kbd>) | Delete the selected point (no modifier) |
| | `modifier` | `0x11` (<kbd>Ctrl</kbd>) | The combination key, and the fast key |
| | `forward` / `back` / `left` / `right` | `0x57` / `0x53` / `0x41` / `0x44` | Move (<kbd>W</kbd> <kbd>S</kbd> <kbd>A</kbd> <kbd>D</kbd>) |
| | `up` / `down` | `0x20` / `0x10` | Move up / down (<kbd>Space</kbd> / <kbd>Shift</kbd>) |
| `motion` | `startSpeed` | `6.0` | Speed when freecam turns on |
| | `minSpeed` / `maxSpeed` | `0.5` / `80.0` | Speed limits for the mouse wheel. `minSpeed` cannot go below `0.5` |
| | `scrollFactor` | `1.25` | Speed change for each wheel step |
| | `fastMultiplier` | `3.0` | Speed multiplier while the fast key is held |
| | `sensitivity` | `0.15` | Mouse look, in degrees for each mouse count |
| | `moveEaseRate` / `lookEaseRate` | `6.0` / `14.0` | How fast smooth motion catches up. Higher is quicker |
| | `pitchLimit` | `89.9` | Highest up or down look angle, in degrees. Maximum `89.9` |
| | `maxFrameDt` | `0.1` | Longest frame time used for motion, in seconds. Maximum `1.0` |
| | `smoothAtStart` | `true` | Smooth motion is on when freecam turns on |
| `ui` | `eventQueueCapacity` | `256` | Size of the queue that carries input to the UI. Range `16` to `4096` |
| | `maxVertices` | `65536` | A UI frame with more vertices is not drawn. Range `4096` to `1048576` |
| `markers` | `enabled` | `true` | Draw the point boxes and path line in the world |
| | `maxShapes` | `4096` | Most shapes drawn at one time. Range `2560` to `8192` |
| | `maxPerSend` | `400` | Most shapes sent to the game in one frame. Range `50` to `800` |
| | `lineWeight` | `2` | `1` or `2`. `2` draws each box edge two times for a bolder line |
| | `frustumDepth` | `1.2` | Length of a camera box, in blocks. Range `0.3` to `5.0` |
| `lookAt` | `enabled` | `true` | Add point can store a target when the crosshair hits a block or entity |
| | `maxDistance` | `256.0` | Length of the look-at ray, in blocks. Range `8` to `512` |
| `screenshot` | `folder` | `%USERPROFILE%\Pictures\Screenshots` | Where screenshots go. Variables are expanded and the folder is created. A blank value uses the default |
| top level | `exitWaitFrames` | `10` | Frames that the exit takes before control returns to your player. Minimum `4` |
| | `telemetry` | `false` | Write frame timing to the log, for testing |

</details>

---

## ❓ FAQ

<details>
<summary><b>The mod does not load.</b></summary>

Check that LeviLamina is the **client** build for your game version (see [Compatibility](#-compatibility)), and that the mod folder is named exactly `mods/alaydriem-freecam/`. The LeviLamina log in the game folder's `logs` folder names the cause.

</details>

<details>
<summary><b>Can I use it on servers and Realms?</b></summary>

The mod is client-side and sends nothing to the server. It does not hide from anti-cheat, and some servers do not allow freecam. Read the server's rules before you use it there.

</details>

<details>
<summary><b>Why can I not see far-away areas?</b></summary>

The camera shows only chunks that the server has sent to your game. Freecam cannot load chunks that the server has not sent.

</details>

<details>
<summary><b>Freecam turned off by itself.</b></summary>

This is a safety exit. Freecam turns off when you take damage, die, change dimension or leave the world, or when the server takes control of the camera. It does not start while the server controls the camera.

</details>

<details>
<summary><b>Can I use a controller?</b></summary>

No. Freecam uses the keyboard and mouse only.

</details>

---

## 🛠️ Building from source

You need Windows 10 or 11 (x64) and [mise](https://mise.jdx.dev). mise installs the pinned versions of xmake and cbindgen. You also need Rust through rustup (the version is pinned in `rust-toolchain.toml`), [cargo-nextest](https://nexte.st), and LLVM with `clang-cl`.

```powershell
git clone https://github.com/alaydriem/levi-freecam
cd levi-freecam
mise trust
mise run preflight   # names any missing tool
mise run build-mod   # output: shim/bin/alaydriem-freecam/
mise run test        # every Rust test
```

<details>
<summary><b>How the code is organised</b></summary>

All the decisions are made in Rust: `freecam/core` (camera motion, input, paths, no game code) and `freecam/ffi` (the C interface). A thin C++ part in `shim/` connects to the game through LeviLamina, applies the camera each frame, and draws the in-world path markers and the egui interface (the playback bar). Run `mise tasks` to see every task.

</details>

---

## 🔗 Related

- [Alaydriem's Mini Map](https://github.com/alaydriem/levi-minimap): a minimap and full world map, for the same game and loader.

## 📄 License and credits

BSD 3-Clause. See [LICENSE](./LICENSE).

Thanks to [LeviLamina](https://github.com/LiteLDev/LeviLamina) and [egui](https://github.com/emilk/egui).
