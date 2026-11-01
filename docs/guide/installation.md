# Installation

iguard is a graphical Polkit authentication agent for Linux desktop sessions. It serves an authentication agent object at `/org/iguard/PolicyKit1/AuthenticationAgent` on the system bus and renders prompts with [Iced](https://iced.rs/) instead of the toolkit your desktop ships with.

## Quick install

The installer downloads a prebuilt release archive from GitHub Releases and installs it for the current user:

```bash
curl -fsSL https://dantescur.github.io/iguard/install.sh | bash
```

It writes `~/.local/bin/iguard` and `~/.config/autostart/iguard.desktop`. The autostart entry starts iguard with your next desktop session; run `iguard` now to start it immediately.

For a system-wide install:

```bash
curl -fsSL https://dantescur.github.io/iguard/install.sh | sudo bash -s -- --system
```

Prebuilt archives are available for x86_64 and aarch64 glibc systems. On musl, build from source.

::: tip Checksum verification
The installer verifies the archive against the `SHA256SUMS` published with the release. That catches corrupt downloads, not a compromised release. If you want to audit the script first, download it and read it before running it.
:::

### Installer options

| Option | Effect |
| --- | --- |
| `--system` | Install to `/usr/local/bin` and `/etc/xdg/autostart` (requires root). |
| `--user` | Install for the current user (default). |
| `--version TAG` | Pin a release tag, for example `v0.1.0`. |
| `--repo OWNER/REPO` | Install from a fork or mirror. |
| `--prefix DIR` (alias `--dir DIR`) | Install the binary into a custom directory. |
| `--no-autostart` | Skip the desktop autostart entry. |
| `--no-verify` | Skip SHA-256 checksum verification. |
| `--uninstall` | Remove the binary and autostart entry for the current scope. |

The installer requires bash. Run it with `--help` for the full option and environment-variable list:

```bash
curl -fsSL https://dantescur.github.io/iguard/install.sh | bash -s -- --help
```

## Requirements

- A Linux desktop session with a running Polkit daemon (`polkitd`)
- A graphical session (X11 or Wayland) — iguard opens a window, so it needs a display
- **Rust 1.85+** if building from source (the crate uses `edition = "2024"`)
- A session bus and a system bus (standard on any systemd or elogind setup)

iguard does **not** need to run as root. Polkit handles privilege escalation; the agent only collects credentials.

## Build from source

Use this if you need a musl build or want to run an unreleased revision.

```bash
git clone https://github.com/Dantescur/iguard
cd iguard
cargo build --release
```

The binary ends up at `target/release/iguard`.

Install it somewhere on your `PATH`:

```bash
install -Dm755 target/release/iguard ~/.local/bin/iguard
```

## Running iguard

Start it inside your graphical session:

```bash
iguard
```

On first run it writes a default configuration to:

```
$XDG_CONFIG_HOME/iguard/config.toml    # usually ~/.config/iguard/config.toml
```

See [Configuration](/guide/configuration) for every available key.

## Autostart

Polkit keeps one agent per session. If your desktop already runs one (`polkit-gnome`, `polkit-kde`, `lxpolkit`, etc.), stop it before starting iguard, or iguard will fail to register.

The installer creates an XDG autostart entry automatically. If you installed manually, create it yourself:

```ini
# ~/.config/autostart/iguard.desktop
[Desktop Entry]
Type=Application
Name=iguard Polkit Authentication Agent
Comment=Authenticate requests for system authorization
Exec=/home/you/.local/bin/iguard
Terminal=false
NoDisplay=true
X-GNOME-Autostart-enabled=true
```

Replace `/home/you` with your home directory, or use `Exec=iguard` if `~/.local/bin` is on your session `PATH`.

### Wayland compositors

If your compositor launches programs from its own config, add iguard there instead of relying on XDG autostart. Use the absolute path (`/home/you/.local/bin/iguard` or `/usr/local/bin/iguard`) if the compositor does not inherit your shell `PATH`.

```
# ~/.config/sway/config
exec iguard
```

```
# ~/.config/hypr/hyprland.conf
exec-once = iguard
```

```
// ~/.config/niri/config.kdl
spawn-at-startup "iguard"
```

### systemd user service (optional)

If you prefer systemd user units over XDG autostart, note that iguard needs `DISPLAY` / `WAYLAND_DISPLAY`, so the unit must be started **after** the graphical session is up. This unit assumes a per-user install; use `/usr/local/bin/iguard` for a system-wide install. A common pattern is a unit pulled in by a target your session activates:

```ini
# ~/.config/systemd/user/iguard.service
[Unit]
Description=iguard Polkit authentication agent
PartOf=graphical-session.target
After=graphical-session.target

[Service]
Type=simple
ExecStart=%h/.local/bin/iguard
Restart=on-failure

[Install]
WantedBy=graphical-session.target
```

Then:

```bash
systemctl --user daemon-reload
systemctl --user enable --now iguard.service
```

> **Note:** `graphical-session.target` is only reliable on systemd setups where your compositor/DE imports the environment. On minimal WMs you may need a `systemd --user` environment import (`dbus-update-activation-environment --systemd WAYLAND_DISPLAY DISPLAY`) or you're better off using XDG autostart.

## Uninstall

Run the installer with `--uninstall` for the scope you installed:

```bash
curl -fsSL https://dantescur.github.io/iguard/install.sh | bash -s -- --uninstall
```

For a system-wide install:

```bash
curl -fsSL https://dantescur.github.io/iguard/install.sh | sudo bash -s -- --uninstall --system
```

This removes the binary and autostart entry. Configuration files are left untouched. If you installed with a custom `--prefix` or `--dir`, pass the same path to the uninstall command.

## Verifying registration

There is no read-only "list agents" call in the Polkit authority API. Check the process and then trigger a prompt:

```bash
pgrep -x iguard
pkexec true
```

If iguard is running and registered, its window appears for the `pkexec` request. Cancel the dialog to avoid running the command.

You can also watch the agent's log output:

```bash
journalctl --user -u iguard.service -f   # if using systemd
# or just watch the terminal where you ran `iguard`
```

## Troubleshooting

| Symptom | Likely cause |
|---------|--------------|
| No window appears on `pkexec` | Another agent is already registered for the session, or iguard was started outside the graphical session (no `DISPLAY`/`WAYLAND_DISPLAY`). |
| `could not register the polkit agent` in stderr | The system bus is unreachable, or another agent holds the name. |
| Installer reports a missing release asset | The requested tag does not have an archive for your architecture yet, or the release is still publishing. Check the release assets. |
| Config changes not applied | iguard polls the file every second; make sure you're editing the path from `iguard`'s `XDG_CONFIG_HOME`, not a root-owned copy. |
| Theme looks wrong | If your theme name is unknown, iguard falls back to `dark`. Check `appearance.theme` spelling against the supported list. |
