# iguard

`iguard` is a graphical [Polkit](https://github.com/polkit-org/polkit) authentication agent for Linux desktop sessions. It displays an authentication prompt when an application requests authorization and returns the response to Polkit.

The iguard source code is licensed under the MIT License; see [LICENSE](LICENSE). The release binary links [`zbus-polkit-agent`](https://crates.io/crates/zbus-polkit-agent), which is licensed under GPL-3.0-or-later, so binary distributions are GPL-3.0-or-later.

## Status

iguard is an early-stage project. It registers itself with Polkit when started inside a graphical user session. The release workflow builds glibc x86_64 and aarch64 archives when a version tag is pushed; downloads are available after that workflow publishes the release. Fedora packaging is also in progress. Do not run iguard alongside another Polkit authentication agent.

## Requirements

- Linux with Polkit and a working system D-Bus
- An active graphical desktop session with a session D-Bus
- Rust and Cargo (edition 2024) when building from source
- [`just`](https://github.com/casey/just) is optional and provides shortcuts for common development commands
- The native libraries required by Iced on your distribution; these depend on whether your desktop uses Wayland, X11, or both

## Install

The installer downloads a prebuilt release archive from GitHub Releases and installs it for the current user:

```sh
curl -fsSL https://dantescur.github.io/iguard/install.sh | bash
```

It installs the binary to `~/.local/bin/iguard` and an autostart entry to `~/.config/autostart/iguard.desktop`. To install system-wide instead:

```sh
curl -fsSL https://dantescur.github.io/iguard/install.sh | sudo bash -s -- --system
```

Run the installer with `--uninstall` to remove the binary and autostart entry for the current scope; configuration files are left untouched. The installer verifies the release archive against the SHA-256 checksum published with the release. Checksums detect corrupt downloads, not a compromised release.

Release binaries target x86_64 and aarch64 glibc systems. musl systems must build from source. The release workflow builds on Ubuntu 22.04 to target glibc 2.35 or newer.

To publish a release, update `version` in `Cargo.toml`, push the matching version tag, and let GitHub Actions build and publish the binaries and checksums:

```sh
git tag v0.1.0
git push origin v0.1.0
```

The tag must match the Cargo version. The workflow publishes archives named for the tag and target, plus `SHA256SUMS`, which the installer downloads from GitHub Releases. The documentation site and installer script are deployed separately by the Pages workflow when changes are pushed to `main`.

## Build and run

```sh
cargo build --release
./target/release/iguard
```

Keep the process running in your desktop session. Starting it from a terminal is useful while evaluating the project because startup and registration errors are printed there.

## Configuration

On first launch, iguard creates a TOML file at:

```text
$XDG_CONFIG_HOME/iguard/config.toml
```

If `XDG_CONFIG_HOME` is unset or relative, the fallback is `~/.config/iguard/config.toml`. Changes are reloaded automatically after saving the file.

The file has sections for:

- `[appearance]`: Iced built-in themes and optional palette colors (`#RGB`, `#RRGGBB`, or `#RRGGBBAA`)
- `[window]`: dimensions, placement, decorations, resizability, stacking, and window controls
- `[behavior]`: focus, Escape/Enter behavior, authentication attempt limit, and optional action details
- `[text]`: prompt labels and status messages
- `[typography]`: font family, weight, sizes, spacing, and padding

The generated configuration includes defaults and comments with accepted theme, font, and placement values. Window dimensions and authentication attempts are bounded by the application.

## Distribution

Fedora is the first distribution target. The RPM installs the executable and a desktop autostart entry, with runtime shared-library requirements detected from the binary. A distro package fits the agent's need to register with Polkit over the system bus from the user's active session. AppImage can be evaluated later, but it does not by itself solve autostart or system integration.

The Fedora RPM spec and desktop autostart entry are in `packaging/fedora/`. On Fedora with RPM and Cargo packaging tools installed, `just fedora-rpm` builds binary and source RPMs from the current checkout. The result is written under `target/rpm/`. The package has been built and its tests passed on Fedora 44; a clean COPR build and real-session install test are still needed. Packaging must not install a Polkit policy that grants privileges; iguard is the authentication UI, while authorization policy remains the responsibility of the system and the requesting applications.

## Development

```sh
just verify
just build
just run
```

The equivalent Cargo commands are `cargo fmt --check`, `cargo check --locked`, `cargo test --locked`, `cargo build --release --locked`, and `cargo run --locked`.
