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
  - The HUD, the hand and the crosshair are hidden while freecam is on.
- **🛤️ Camera paths**
  - Create simple camera paths for playback in-game to take beautiful recordings in your world with easing, with simple overlay paths
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
| Mouse | Look | Freecam on |
| Mouse wheel | Change the speed | Freecam on |
| <kbd>Ctrl</kbd> + <kbd>[</kbd> | Smooth motion on / off | Freecam on |
| <kbd>Ctrl</kbd> + <kbd>B</kbd> | Add a path point | Freecam on, path not playing |
| <kbd>Ctrl</kbd> + <kbd>Z</kbd> | Remove the last path point | Freecam on, path not playing |
| <kbd>Ctrl</kbd> + <kbd>Enter</kbd> | Play or stop the path | Freecam on, 2 or more points |

Key combinations work only when no game menu or chat is open.

> [!TIP]
> Every key is a setting in `config.json`, written as a [Windows virtual-key code](https://learn.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes). <kbd>Ctrl</kbd> is the `modifier` key for every combination, and it is also the fast key.

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
| | `undoPoint` | `0x5A` (<kbd>Z</kbd>) | With the modifier: remove the last path point |
| | `play` | `0x0D` (<kbd>Enter</kbd>) | With the modifier: play or stop the path |
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

All the decisions are made in Rust: `freecam/core` (camera motion, input, paths, no game code) and `freecam/ffi` (the C interface). A thin C++ part in `shim/` connects to the game through LeviLamina, applies the camera each frame, and draws the path overlay with Dear ImGui. Run `mise tasks` to see every task.

</details>

---

## 🔗 Related

- [Alaydriem's Mini Map](https://github.com/alaydriem/levi-minimap): a minimap and full world map, for the same game and loader.

## 📄 License and credits

BSD 3-Clause. See [LICENSE](./LICENSE).

Thanks to [LeviLamina](https://github.com/LiteLDev/LeviLamina) and [Dear ImGui](https://github.com/ocornut/imgui).
