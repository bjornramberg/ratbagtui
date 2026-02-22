# ratbagtui

A terminal UI device manager for configurable mice on Linux, built on top of [libratbag](https://github.com/libratbag/libratbag).

Developed and tested on a Logitech MX Vertical, but should supports any mouse recognized by `ratbagd`.

---

## Features

- Read and set DPI from the device's supported values
- Remap mouse buttons
- Test mode — click buttons and see what they're mapped to in real time
- Multi-device support via ratbagd's D-Bus interface
- Single native binary, no runtime dependencies beyond libratbag

---

## Requirements

- `libratbag` — must be installed and `ratbagd` running
- Linux with `udev` and `systemd`

### Install libratbag

**Arch Linux:**
```bash
sudo pacman -S libratbag
```

**Ubuntu/Debian:**
```bash
sudo apt install ratbagd
```

**Fedora:**
```bash
sudo dnf install libratbag
```

### Start ratbagd

ratbagd runs as a systemd service. Enable and start it:

```bash
sudo systemctl enable --now ratbagd
```

Verify your device is detected:

```bash
ratbagctl list
```

---

## Installation
<!-- Not yet in place...
### From binary (GitHub Releases)

Download the latest binary from the [Releases](../../releases) page:

```bash
curl -Lo ratbagtui https://github.com/bjornramberg/ratbagtui/releases/latest/download/ratbagtui-x86_64-unknown-linux-gnu
chmod +x ratbagtui
sudo mv ratbagtui /usr/local/bin/
```
-->
### From source

Requires Rust (stable):

```bash
git clone https://github.com/bjornramberg/ratbagtui
cd ratbagtui
cargo build --release
sudo cp target/release/ratbagtui /usr/local/bin/
```

---

## Setup

### udev rules (required)

Without a udev rule, ratbagtui must be run with `sudo`. To run as a normal user, install the included udev rule:

```bash
sudo cp pkg/70-libratbag.rules /etc/udev/rules.d/
sudo udevadm control --reload-rules
sudo udevadm trigger
```

Then unplug and replug your mouse/receiver.

> **Note:** The Arch Linux `libratbag` package does not ship udev rules. The rules file must be installed manually or via this package.

### input group (required for test mode)

Test mode reads raw HID input from `/dev/hidraw*`. Your user must be in the `input` group:

```bash
sudo usermod -aG input $USER
```

Then **fully log out and back in** to your desktop session. Restarting a terminal emulator is not sufficient — the group change requires a new session.

> **Note:** You can verify it's active with `id` — look for `input` in the groups list. If it's missing after a terminal restart, use `newgrp input` as a workaround in the current shell.

---

## Usage

```
ratbagtui
```

### Keybindings

| Key | Action |
|-----|--------|
| `Tab` | Switch between DPI and Buttons panel |
| `↑` / `k` | Navigate up |
| `↓` / `j` | Navigate down |
| `Enter` | Apply selected DPI / open button editor |
| `t` | Enter test mode |
| `Esc` | Close popup / exit test mode |
| `q` | Quit |

### Test Mode

Press `t` to enter test mode. Click any mouse button and the display will show which button was detected and what action it is currently mapped to. This is useful for verifying that button remaps have taken effect.

Test mode reads directly from the hidraw device (e.g. `/dev/hidraw6`) rather than the evdev input node. This is necessary because Wayland compositors hold an exclusive grab on `/dev/input/event*` nodes, making them inaccessible to other processes.

---

## Known Limitations

### Key remapping is not supported

The Logitech MX Vertical (and most Logitech mice) only support button-to-button remapping in firmware. ratbagd will accept key remapping commands without error, but the hardware silently ignores them. ratbagtui only exposes button actions in the remapping UI for this reason.

### Wayland input grab

Wayland compositors exclusively grab `/dev/input/event*` devices, so test mode cannot use the standard evdev interface. ratbagtui works around this by reading from the raw HID device node directly (`/dev/hidraw*`), which is not grabbed by the compositor.

### sudo requirement without udev rules

The Arch Linux `libratbag` package does not install udev rules, so the hidraw device nodes default to root-only access. The included `pkg/70-libratbag.rules` file addresses this. See the Setup section above.

### newgrp workaround

If you have just added yourself to the `input` group, you need a full session logout/login for it to take effect system-wide. Using `newgrp input` in a shell will apply it to that shell only.

### ratbagd must be running

ratbagtui communicates with the mouse via ratbagd over D-Bus. If ratbagd is not running, ratbagtui will exit with "No devices found." Ensure the service is active:

```bash
systemctl status ratbagd
```

### Single profile support

ratbagtui currently reads and writes to the active profile only. Mice with multiple profiles (e.g. profile switching via a dedicated button) will only have their active profile managed.

---

## Architecture

```
ratbagtui
├── src/
│   ├── main.rs          # TUI, event loop, app state
│   ├── dbus/
│   │   ├── mod.rs
│   │   ├── proxies.rs   # Raw zbus D-Bus proxy traits
│   │   └── device.rs    # Friendly structs wrapping the proxies
│   └── input.rs         # hidraw reader for test mode
├── pkg/
│   ├── PKGBUILD         # Arch Linux AUR package
│   └── 70-libratbag.rules  # udev rules
```

**D-Bus stack:**

```
ratbagtui  →  D-Bus  →  ratbagd  →  HID protocol  →  mouse hardware
```

ratbagtui never speaks to the hardware directly for configuration — everything goes through ratbagd, which means any mouse supported by libratbag should work.

---

## Contributing

Issues and PRs welcome. If you have a mouse that doesn't work correctly, opening an issue with the output of:

```bash
busctl introspect org.freedesktop.ratbag1 /org/freedesktop/ratbag1
ratbagctl list
ratbagctl "<your device name>" profile active get
```

...will help a lot.

---

## License
MIT