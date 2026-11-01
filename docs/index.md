---
layout: home

hero:
  name: iguard
  text: System permission, rendered properly.
  tagline: A small, native Polkit authentication agent for Linux desktops. Rust and Iced, a single TOML file, and no toolkit baggage.
  actions:
    - theme: brand
      text: Install iguard
      link: /guide/installation
    - theme: alt
      text: How it works
      link: /guide/how-it-works

features:
  - title: Native, not bolted on
    details: One Iced window. No GTK, Qt, or Electron runtime to drag into your session.
  - title: Edit it live
    details: Change theme, type, copy, or attempt limits in config.toml. iguard reloads while it runs.
  - title: Small on purpose
    details: It registers one D-Bus name and draws one dialog. Authorization policy stays with Polkit.
  - title: Knows your session
    details: Starts from XDG autostart or a systemd user unit, with environment checks before it registers.
---

## What happens when something asks for permission

1. A process asks Polkit for an action that needs authentication.
2. Polkit routes the request to the session's registered agent — iguard.
3. iguard opens one dialog, collects the password, and returns the result.
4. Polkit decides. iguard never grants anything by itself.

[Read the full flow →](/guide/how-it-works)

## Why another agent?

Most Polkit agents are part of a larger desktop toolkit. iguard is not. It is a single Rust binary with one job: show the prompt, collect the response, and get out of the way.

- **No toolkit lock-in.** The dialog is drawn with Iced, so it looks the same on GNOME, KDE, sway, or a bare X session.
- **No policy changes.** iguard is the UI. Polkit still owns every authorization decision.
- **No restart loop.** Edit `config.toml` and the next prompt uses the new settings; window changes apply live.

[Configuration reference →](/guide/configuration)

::: tip One agent per session
Polkit allows one authentication agent per session. Stop any existing agent before starting iguard.
:::
