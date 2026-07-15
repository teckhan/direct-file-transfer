# Direct File Transfer

Who needs a pen drive? Share files between devices on the same network — no cloud, no accounts, no size limits beyond your disk.

One machine runs the desktop app (the **host**). Every other device — phone, tablet, laptop — just opens a URL in its browser (the **guest**). Nothing to install on the guest side.

## How it works

- The host app embeds a small HTTP server (port `8080`) that serves both the shared files and the guest web page itself.
- Guests open `http://<host-local-ip>:8080` in any browser to download shared files or upload files back to the host.
- Live updates (files added/removed) are pushed to guests over Server-Sent Events, so everyone's list stays in sync.
- Files never leave your network. There is no third-party relay.

## Usage

1. Launch the app on the host machine.
2. Add files by drag-and-drop or with **Add files**.
3. On another device, scan the QR code shown in the app (or type the **Local Ip** URL into a browser).
4. Download files from the list, or upload files to the host — uploads land in the host's save folder (Desktop by default, configurable in the app).

### A note on the public IP

The app also displays your **public** IP. That address only works for people outside your network if your router forwards TCP port `8080` to the host machine (and your ISP doesn't block it / use CGNAT). Without port forwarding, treat this app as LAN-only — which is the recommended way to use it anyway, since the server has no authentication: anyone who can reach the port can download the shared files and upload files to the host.

## Install

Grab the installer for your platform from the [releases page](../../releases), or build from source (below).

## Development

Prerequisites: [Node.js](https://nodejs.org) + [Yarn](https://classic.yarnpkg.com), the [Rust toolchain](https://rustup.rs), and the [Tauri prerequisites](https://tauri.app/start/prerequisites/) for your OS.

```sh
yarn install
yarn tauri dev      # run the desktop app (also builds the guest page)
```

Other useful commands:

```sh
yarn typecheck                      # vue-tsc over the frontend
yarn build                          # build the guest page into dist/
cargo test --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml
yarn tauri build                    # produce installers for this machine
```

> **Note:** the Rust build embeds `dist/` as a resource, so run `yarn build` at least once before any `cargo` command that compiles the app.

Releases are built by the [Release workflow](.github/workflows/release.yml) when a `v*` tag is pushed.

## Architecture

See [AGENTS.md](AGENTS.md) for a code-level tour: the host/guest split, the HTTP + SSE protocol, and project conventions.
