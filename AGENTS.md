# Agent & contributor guide

Tool-agnostic notes for anyone (human or AI coding agent) working on this repo.

## What this is

A Tauri 2 desktop app for LAN file sharing. One Vue 3 frontend serves **two roles from the same bundle**:

- **Host** — the Tauri desktop app. Detected at runtime via `"__TAURI__" in window` in `src/App.vue`, which renders `src/pages/Host.vue`.
- **Guest** — any browser that opens `http://<host-ip>:8080`. The embedded server serves the same `dist/` bundle; without `__TAURI__`, `src/pages/Guest.vue` renders.

## Layout

```
src/                    Vue 3 + TypeScript + Tailwind 4 frontend
  pages/Host.vue        host UI (talks to Rust via Tauri invoke/events)
  pages/Guest.vue       guest UI (talks to the HTTP API via axios + SSE)
  components/ui/        shadcn-vue style primitives (reka-ui based) — generated, keep edits minimal
  components/organisms/ app-specific composites (tanstack-table wrappers)
  types/File.ts         shared view-model types
src-tauri/              Rust backend
  src/lib.rs            Tauri setup, commands, config persistence, single-instance
  src/api.rs            embedded actix-web server (port 8080) + shared-file state
  src/api/broadcast.rs  SSE broadcaster for guest live updates
```

## The two communication channels

**Host UI ⇄ Rust (Tauri):**

- Commands: `add_file`, `remove_file`, `clear_files`, `get_local_ip`, `get_public_ip`, `get_server_error`, `get_save_dir`, `set_save_dir`, `reveal_in_folder`.
- Events (Rust → host UI): `file-added`, `file-removed`, `cleared-all`, `file-received` (a guest uploaded something), `server-error`.

**Guests ⇄ embedded server (HTTP on `0.0.0.0:8080`):**

- `GET /list` → `[{ id, file_name, size }]`
- `GET /dl/{id}` → streamed download (supports Range)
- `GET /dl` → single file, or zip (`Stored` compression) when multiple files are shared
- `POST /upload` → multipart field `file`; saved into the configured save dir, collisions renamed to `name (n).ext`
- `GET /events` → SSE; messages are `{ action, payload }` where `payload` is a JSON **string**. Actions: `connected`, `file-added`, `file-removed`, `all-files-cleared`.

If you change an event or endpoint shape, update **both** pages and this file.

## Commands

```sh
yarn install
yarn build          # build frontend into dist/ — REQUIRED before cargo builds (see gotchas)
yarn typecheck      # vue-tsc --noEmit
yarn tauri dev      # run the desktop app

cd src-tauri
cargo fmt
cargo clippy
cargo test
```

## Gotchas

- `tauri::generate_context!` embeds `../dist` as a resource, so **`dist/` must exist before compiling the Rust crate** (build the frontend first). CI does this too.
- Shared state in `api.rs` is module-level (`FILE_LIST`, `SAVE_DIR`, `BROADCASTER` behind `LazyLock`), because it's reached from both Tauri command threads and actix handlers. Rust tests touching that state share it across tests.
- Async spawns that can run outside the actix runtime must use `tauri::async_runtime::spawn` (see `broadcast.rs`) — `actix_web::rt::spawn` panics off the actix `System`.
- Filename handling must go through `std::path::Path` (never regex/string-splitting on `/`) so Windows paths work.
- The server intentionally has **no authentication** (LAN tool). Don't add features that assume trusted input from guests; uploads already sanitize to the final path component.
- `tauri-plugin-single-instance` must stay the **first** plugin registered.
- Frontend formatting is 4-space / prettier-ish; Rust is `cargo fmt` defaults.

## Verifying changes

There is no e2e harness. To sanity-check by hand: `yarn tauri dev`, add a file, then open `http://localhost:8080` in a browser tab — it acts as a guest (list, download, upload, and live SSE updates should all work).
