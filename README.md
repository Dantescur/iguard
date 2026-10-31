# iguard

`iguard` is a graphical [Polkit](https://github.com/polkit-org/polkit) authentication agent for Linux desktop sessions. It displays an authentication prompt when an application requests authorization and returns the response to Polkit.

iguard is licensed under the MIT License; see [LICENSE](LICENSE).

## Status

iguard is an early-stage project. It currently builds from source and registers itself with Polkit when started inside a graphical user session. It does not yet include distro packages, a desktop autostart entry, or an installer. Run it manually while evaluating it; do not run it alongside another Polkit authentication agent.

## Requirements

- Linux with Polkit and a working system D-Bus
- An active graphical desktop session with a session D-Bus
- Rust and Cargo (edition 2024)
- [`just`](https://github.com/casey/just) is optional and provides shortcuts for common development commands
- The native libraries required by Iced on your distribution; these depend on whether your desktop uses Wayland, X11, or both

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

Fedora is the first planned distribution target. The initial package will be an RPM that installs the executable and a desktop autostart entry, with package metadata declaring the runtime requirements. A distro package is a better initial fit than a sandboxed bundle because the agent needs to register with Polkit over the system bus from the user's active session. AppImage can be evaluated later, but it does not by itself solve autostart or system integration.

RPM packaging is not implemented yet. Before publishing packages, the project needs a stable repository URL, a desktop entry, documented Fedora build dependencies, and a real-session installation test. Packaging must not install a Polkit policy that grants privileges; iguard is the authentication UI, while authorization policy remains the responsibility of the system and the requesting applications.

## Development

```sh
just verify
just build
just run
```

The equivalent Cargo commands are `cargo fmt --check`, `cargo check --locked`, `cargo test --locked`, `cargo build --release --locked`, and `cargo run --locked`.
