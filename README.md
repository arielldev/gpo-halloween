[![Discord](https://img.shields.io/badge/Discord-Join%20Server-7289da?style=for-the-badge&logo=discord&logoColor=white)](https://discord.gg/unPZxXAtfb)

# 🎃 GPO Halloween v1.0 - GUIDE

**💬 Join our Discord server:** https://discord.gg/unPZxXAtfb

**🎣 Looking for the fishing macro?** [GPO Autofish](https://github.com/arielldev/gpo-fishing)

## What is this?

An **open-source** macro for the Grand Piece Online Halloween event. It walks from house to house with click to move, presses **E** at every door, and leaves your private server before the NPCs spawn at 5 minutes. Then it rejoins with your server code and starts again.

- ✅ **Fully open source** - You can see and verify all the code
- ✅ **No viruses** - Clean, transparent, and safe
- ✅ **No false positives** - The server timer has to be read several times in a row, in step with the real clock, before it ever triggers a rejoin

**🛡️ Concerned about safety?** The whole source is here. Build it yourself with the steps below and compare. Antivirus heuristics dislike programs that send mouse input and read the screen; that is what a macro does.

---

**Features:**

- **🏠 House Route** - Records your click-to-move path from spawn to every house, TinyTask style, and plays it back
- **🚪 Auto Interact** - Holds **E** at each house, then waits 5 seconds before moving on
- **⏱️ Server Timer Watch** - Reads the private server timer in the bottom right with Windows OCR and stops at **4:50**, mid-step if needed
- **🔁 Auto Rejoin** - Leaves to the lobby, pastes your server code, picks the server type and server, and joins
- **📍 Fixed Spawn** - Every run starts from the spawn you set at the island NPC, so clicks always land in the same place
- **🎥 Fixed Camera** - Third person from above at the widest zoom, set the same way before recording and every run
- **🍬 Candy Bag** - Presses your bag slot (default `6`) after the camera setup so the doors open
- **📌 HUD pill + tray icon** - A small always-on-top pill on the Roblox window with the timer, houses and rejoins
- **🖱️ On-screen editor** - Lay out the timer area and pick click points directly over the game
- **🛟 Watchdog** - Leaves to the lobby and restarts if a step hangs
- **🔔 Discord webhook** - Stats, recent activity and the community link posted every N routes, plus start/stop and problem alerts
- **⬆️ Auto update** - Checks GitHub Releases on launch; one click installs the new version
- **💾 Presets + JSON export** - Save setups, and export or import the full settings JSON
- **📤 Share sequences** - Export any single sequence (Lobby, Macro, Leave to lobby) to a file and import it on another PC
- **⌨️ Global hotkey support** (F1 to F8, all rebindable)

## 🚀 Key Features

### ⏱️ Server Timer Watch

- **OCR Detection**: Reads the bottom-right timer using Windows text recognition
- **No False Positives**: A rejoin only triggers after 3 readings in a row that advance with real time (4:48 → 4:49 → 4:50). A single misread like `1:50` → `4:50` is ignored, and a stuck number (like `Version 13.50` under the timer) never counts
- **Strict Parsing**: Digits and `:` first; OCR letter fixes (`O`→`0`, `l`→`1`) are only tried if that fails
- **Only While In-Server**: The watch is armed after joining and disarmed while leaving and rejoining
- **Backup Clock**: If the timer can't be read, it counts from the moment you joined

### 🏠 House Route

- **Click to Move**: Every left click on the game is saved with its exact timing
- **Interact Steps**: Pressing **E** while recording becomes an Interact step (hold E, wait 5 s)
- **Editable**: Retime, reorder, re-pick or delete any step afterwards. Picking shows the whole route numbered on screen
- **Recorder Safety**: Only clicks on the Roblox window are recorded (not the HUD or panel), and only keys pressed while Roblox is focused

### 🔁 Auto Rejoin

- **Leave to lobby**: 2 binds the moment the 5:00 listener fires: menu toggle (opens the accordion) → **Main menu**
- **Lobby**: Click to proceed → **Private server join** → **Paste** your server code → **Regular** twice (the first click confirms the code) → **First Sea** server → joins
- **F1 run order**: Binds › F1 run order lists the sequences F1 runs, top to bottom, then repeats (default Lobby → Macro → Leave to lobby). Add, reorder, switch off or remove blocks like Macro steps. If the timer hits 4:50 during an in-server block it jumps to Leave to lobby
- **Buy loop**: A Buy sequence you build yourself (e.g. walk to the shop NPC, scroll the list, pick the item, accept). Set *Replace the Macro every N routes* and every Nth Macro becomes a Buy run, with Leave and Lobby unchanged
- **Walk time**: Each click-to-move step waits its own time (recorded, or set by you) before the next step, e.g. pressing E
- **Change the code any time**: The Paste step uses `{code}`, so it always pastes the current code

## Installation

### 🚀 Easy Installation (Recommended)

1. **Download** the latest `GPO Halloween_x.y.z_x64-setup.exe` from Releases or our [Discord](https://discord.gg/unPZxXAtfb)
2. **Run it** - No admin needed
3. **Launch GPO Halloween** - The panel opens; the HUD appears once Roblox is running

Requires Windows 10 1809 or newer. Windows OCR needs an English language pack, which is present on nearly every install. Home › Get started tells you if it is missing.

### 🔧 Build the installer yourself

Requirements: [Node.js 20+](https://nodejs.org) and [Rust](https://rustup.rs). WebView2 is already on Windows 11.

1. **Download the repository** as ZIP and extract it, or clone it
2. **Double-click `MakeItExe.bat`** - It installs packages, builds the app and opens the folder with the installer
3. **Run the installer** it produced, same as the one from Releases

For development: `npm install` then `npm run app:dev`. Tests: `cd src-tauri && cargo test`.

## 🎮 Quick Start Guide

Everything below is in **Home › Get started**, with the real inputs right there.

### Ready out of the box

The release ships with a complete, tested setup: Lobby rejoin, house route, Leave to lobby, Buy loop (every 5 routes), timer box, camera and candy bag. You only need to:

1. **Paste your private server code** in Home › Get started
2. **Tick Click to Move and Spawn set** once you've done them in game
3. **Check your candy bag slot** (default `1`) matches your hotbar

Everything is relative to a maximized Roblox window, and the macro keeps Roblox maximized for you.

### Before you start

- **Click to Move**: Roblox menu (`Esc`) › Settings › Movement Mode › **Click to Move**. Shift Lock off
- **Spawn**: Talk to the spawn NPC on the island with the Halloween houses and set your spawn there
- **Private server**: Required. Copy your private server code
- **Window size**: The macro keeps Roblox maximized in a window (not fullscreen) automatically, so recorded clicks and the timer area always line up

### First Time Setup

1. **Launch**: Open Roblox, join your private server, then open GPO Halloween. Get started shows the window as detected
2. **Server code**: Paste it into Home › Get started › Private server code
3. **Leave to lobby**: Pick the menu toggle button (opens the accordion) and the **Main menu** button. Test with `F6`
4. **Lobby**: Click to proceed is already set to the screen center. Pick **Private server join**, the code box, **Regular** and the **First Sea** server. Test with `F7` from the title screen
5. **Macro**: Stand at spawn on a fresh load, press **Record**. The camera zooms out first. Click to walk to each house and press `E` at every door. Press `F5` to finish. Test with `F8`
6. **Timer area**: Press `F2` and lay out the box over the bottom-right timer. The box shows the live read while you drag. Press **Read now** to confirm
7. **Farm**: Stand at spawn and press **F1** or the HUD play button

### Discord Webhook Setup

1. **Create a webhook**: Discord › channel settings › Integrations › Webhooks › New webhook › Copy URL
2. **Paste it** in Settings › Discord and press **Test**
3. **Pick how often**: progress every N routes (stats, recent activity and the community link), plus start/stop and problem alerts

### Hotkeys

- **F1**: Start/stop the macro
- **F2**: Edit layout (server timer OCR area)
- **F3**: Emergency stop and exit
- **F4**: Hide/show the HUD
- **F5**: Record / finish recording
- **F6**: Test run Leave to lobby
- **F7**: Test run Lobby
- **F8**: Test run Macro
- **Note**: All hotkeys work without admin privileges and can be rebound in Settings

---

## 🧾 Settings JSON

Settings live in `%APPDATA%\gpo-halloween\settings.json`. **Settings › Defaults & JSON › Export** writes `settings-export.json` next to it, and the copy button puts the JSON on your clipboard. To make your setup the built-in defaults, replace `src-tauri/defaults.json` with it and rebuild.

Every point and area is **relative to the Roblox window** (0 to 1).

| Path | Meaning |
|------|---------|
| `setup.click_to_move`, `setup.spawn_set` | Setup checklist ticks |
| `setup.auto_maximize` | Keep Roblox maximized in a window: restores/maximizes it (and leaves fullscreen with F11) on start and whenever it comes back to the front (default on) |
| `server.code` | **Required.** Private server code, pasted wherever a step says `{code}` |
| `server.after_join_wait_ms` | Wait after joining for the server to load |
| `keys.interact` | Key held at each house (`e`) |
| `keys.equip` | Candy bag slot pressed once after the camera setup (`6`, empty = off) |
| `camera.enabled`, `out_steps`, `in_steps`, `step_delay_ms` | Fixed camera zoom, done before recording and every run |
| `camera.tilt_px` | Right-drag down in pixels to look from above (600 hits the steepest angle; 0 = off) |
| `lobby.steps`, `macro.steps`, `leave.steps`, `buy.steps` | The sequences |
| `run_order` | What F1 runs, in order, e.g. `[{"seq":"lobby"},{"seq":"macro"},{"seq":"leave"}]`; `"enabled": false` skips a block |
| `buy.every` | Replace the Macro with Buy after every N routes (0 = off) |
| `macro.house_wait_ms` | Wait after each Interact (5000 = 5 s) |
| `macro.interact_hold_ms` | How long E is held |
| `macro.start_delay_ms` | Pause after the camera reset, before the route |
| `macro.on_finish` | `rejoin`: leave and rejoin as soon as the route ends (default). `repeat` reruns in the same server (JSON only) |
| `timer.enabled` | Safety stop at the timer limit: leave, rejoin, start again from spawn (default on) |
| `timer.region` | Box around the timer |
| `timer.mode` | `elapsed` (counts up) or `remaining` (counts down) |
| `timer.stop_at_s` | Leave at this time (290 = 4:50) |
| `timer.confirm_reads` | Readings in a row, in step with real time, needed to stop |
| `timer.fallback`, `fallback_after_misses`, `fallback_stop_s` | Backup clock when the timer can't be read |
| `recording.*` | Target, replace/append, record keys, camera first, collapse typing, rounding |
| `hotkeys.*` | Global hotkeys |
| `watchdog.*` | Restarts from Leave to lobby if a step hangs |
| `webhook.enabled`, `url`, `every_routes`, `start_stop`, `errors` | Discord webhook: progress every N routes, start/stop and problem alerts |
| `auto_update` | Check GitHub Releases for a newer version on launch (default on) |

Each step has a `kind`, a `wait_ms` (delay after it) and an optional `note`:

```json
{ "kind": "click", "point": { "x": 0.52, "y": 0.61 }, "button": "left", "wait_ms": 1850 }
{ "kind": "interact", "wait_ms": 0 }
{ "kind": "paste", "text": "{code}", "wait_ms": 800 }
{ "kind": "key", "key": "Escape", "hold_ms": 60, "wait_ms": 500 }
{ "kind": "type", "text": "hello", "wait_ms": 300 }
{ "kind": "scroll", "amount": -10, "point": { "x": 0.5, "y": 0.6 }, "wait_ms": 200 }
{ "kind": "wait", "wait_ms": 8000, "note": "main menu loading" }
```

A click with `"point": null` still needs picking, and the macro won't start until every one is set.

---

## 🔧 Troubleshooting

- **HUD not showing**: It only appears while Roblox is running and not minimized. Press F4 if you hid it
- **Hotkeys not working**: Another app may own the key (for example GPO Autofish uses F1 to F4 too). The macro then listens for the key directly and marks it **taken** in Settings › Hotkeys. If it still doesn't react, close the other app or rebind
- **Character walks to the wrong place**: Re-record the Macro from a fresh load at spawn, with the same window size and Click to Move on
- **Timer not read**: Press F2 and make the box tighter around the timer. Home › Server timer › OCR sees shows exactly what the OCR reads
- **Rejoin fails**: Run Lobby with F7 from the main menu and check each click. Make sure the server code is set
- **Logs**: Settings › Data folder › `logs/gpo-halloween.log` (one file, started fresh each launch)

---

## 📁 Project Structure

```
src-tauri/src/
├── core/platform/       # OS traits (window, capture, input, clipboard, OCR) + Windows implementations
├── core/timer.rs        # Clock parsing and the consistency filter
├── bot/                 # Cycle machine, actions, recorder, timer watch, watchdog, session stats
├── config.rs            # Settings, steps, presets, JSON export
└── commands.rs          # Tauri command surface
src/
├── windows/             # Hud, Panel, Overlay, Guide
├── pages/               # Home (with Get started), Binds, Settings
└── components/          # Step list, timer area, shared UI pieces
```

## 🤝 Contributing

This is an open-source project! Feel free to:

- Report bugs and issues
- Suggest new features
- Submit pull requests
- Join our Discord community

**💬 Discord:** https://discord.gg/unPZxXAtfb

**🎣 Also check out:** [GPO Autofish](https://github.com/arielldev/gpo-fishing), the open-source GPO fishing macro

## License

MIT. See [LICENSE](LICENSE).
