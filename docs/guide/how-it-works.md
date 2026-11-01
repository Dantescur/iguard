# How it works

iguard is a thin bridge between Polkit and an Iced GUI. It runs two nearly independent halves that communicate over channels.

## The two halves

```
┌──────────────────────────────┐        ┌──────────────────────────────┐
│  Polkit side (agent.rs)      │        │  GUI side (app.rs)           │
│                              │        │                              │
│  zbus-polkit-agent           │        │  iced::daemon                │
│  polkit_agent_instance()     │        │  single dialog window        │
│  PolkitAgentSession          │        │  prompts, submits, cancels   │
│  on a dedicated thread +     │        │                              │
│  its own Tokio runtime       │        │                              │
└──────────┬───────────────────┘        └──────────▲───────────────────┘
           │                                       │
           │  UiRequest (unbounded mpsc)           │
           └───────────────────────────────────────┘
                       agent → UI

                          ResponderMap
           ┌───────────────────────────────────────┐
           │  HashMap<cookie, oneshot::Sender>     │
           │  UI → agent (password / cancel)       │
           └───────────────────────────────────────┘
```

- **`agent.rs`** spawns a dedicated OS thread (`"iguard-polkit"`), builds a small multi-threaded Tokio runtime (`worker_threads = 2`), and runs the `zbus-polkit-agent` loop there. This keeps D-Bus work off the GUI thread entirely.
- **`app.rs`** is the Iced daemon. It owns the window, the password field, and the lifecycle of a single dialog. It receives `UiRequest` values from the agent and replies through `oneshot` senders stored in a shared map keyed by the Polkit cookie.

The two sides share nothing except the channels and a `Config` behind an `Arc<RwLock<Config>>` (so config reloads on the GUI side are visible to the agent side, e.g. for `max_attempts` and localized strings).

## Registration

On startup, `agent::spawn` builds a polkit agent instance:

```rust
polkit_agent_instance(
    move || Agent::new(ui, responses, config),
    authenticate,
    cancel_authentication,
)
```

and connects it to the system bus at:

```
/org/iguard/PolicyKit1/AuthenticationAgent
```

`connect` returns once the agent is registered as the session agent for the caller's session. From that point on, Polkit routes authorization requests to iguard.

If `connect` fails (e.g. another agent is registered), the error is printed to stderr and the thread exits; the GUI keeps running but never receives requests.

## The authentication flow

When something asks Polkit for an action that requires authentication, Polkit calls iguard's `authenticate` callback. Step by step:

1. **`authenticate`** receives the action ID, message, details, cookie, and a list of candidate identities.
2. iguard takes the **first Unix user identity** and opens a `PolkitAgentSession` for it.
3. It reads the current config (`max_attempts`, localized `Text`) under the shared lock.
4. It sends **`UiRequest::Begin { info }`** to the GUI. The GUI opens a window and shows a "waiting" state.
5. A **cancellation `oneshot`** is registered in the `cancellations` map keyed by cookie.
6. **`run_session`** drives the polkit session in a loop:
   - It `select!`s between the session's next message and the cancellation signal (cancellation is `biased`, so it wins ties).
   - `SessionMessage::Request` → a password prompt. iguard registers a `oneshot` responder in the `ResponderMap` and sends **`UiRequest::Prompt`** to the GUI.
   - `SessionMessage::Info` / `::Error` → forwarded as **`UiRequest::Notice`**.
   - `SessionMessage::Complete` → the attempt finished.
7. The GUI collects the password and sends **`UiResponse::Password(...)`** (or **`UiResponse::Cancel`**) back through the `oneshot`.
8. On `Password`, iguard calls `session.response(...)`. On `Cancel`, it returns `PolkitError::Cancelled`.
9. When the session completes:
   - **success** → `UiRequest::Close { success: true }`, GUI closes the window.
   - **failure** → if `attempts < max_attempts`, `session.restart()` and loop again with a retry notice; otherwise `Close { success: false }` and `PolkitError::Failed`.

## Cancellation

There are three ways an authentication gets cancelled, and they all converge:

| Trigger | Path |
|---------|------|
| User clicks **Cancel** | GUI sends `UiResponse::Cancel` → `run_session` returns `PolkitError::Cancelled` |
| User presses **Esc** (if `behavior.close_on_escape`) | Same as Cancel |
| Window is closed | `Message::WindowClosed` → `cancel_dialog` sends `UiResponse::Cancel` |
| Polkit itself cancels (`cancel_authentication`) | `cancellations[cookie].send(())` → `select!` branch in `run_session` fires |

In every case the agent sends a final `UiRequest::Close { success: false, error }` so the GUI reliably tears down the dialog, and the responder map entry is removed so nothing leaks.

`UiRequest::Close` is also idempotent from the GUI's perspective: it only acts if the cookie matches the current dialog, so late messages from a previous authentication can't close a newer one.

## Config hot-reload

Both halves read the same `Arc<RwLock<Config>>`:

- The **GUI** side (`app.rs`) holds a `config_path` and the last-seen `SystemTime`. Every second (`Message::Tick`) it stats the file. If the mtime changed and the file parses, it swaps in the new `Config`, updates the shared copy, recomputes the theme, and diffs the `[window]` section to apply window changes (resize, decorations, level, or reopen when placement/closeable/minimizable change).
- The **agent** side (`agent.rs`) reads the shared config at the start of each `authenticate` call, so a reload between two authentications affects the next one (attempt limit, localized strings).

A parse error is logged and the previous config stays in effect — editing the file never breaks a running dialog.

## Why a separate thread and runtime?

Iced's daemon already runs a Tokio runtime (via the `tokio` feature), so in principle the agent could share it. iguard doesn't, for isolation:

- D-Bus dispatch and session handling are bursty and shouldn't compete with rendering.
- If the polkit side panics, the GUI keeps running and the failure is visible in stderr rather than tearing down the window.
- The agent's `run_session` future can be cancelled cleanly without touching Iced's executor.

The cost is one extra OS thread and a two-worker runtime, which is negligible for a dialog that lives for a few seconds at a time.
