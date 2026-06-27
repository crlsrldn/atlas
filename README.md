# Project Atlas

Project Atlas is a local-first Stremio companion that ranks playable streams across configured providers, explains the best match, and exposes a desktop Smart Play surface.

## Supported Providers

Atlas currently supports:

- TorBox for cached torrent verification and playback resolution.
- Real Debrid for cached torrent verification and playback resolution.
- Gemini for optional AI catalog recommendations.
- Appwrite for backend-owned preferences and telemetry storage.

Provider API keys are never returned by the public preferences API. On macOS desktop builds, Atlas stores provider secrets in Keychain under `com.cindrallabs.atlas` and persists only non-secret preferences to disk or Appwrite.

## Local Setup

Requirements:

- Rust stable
- Node.js 20
- npm
- macOS with Xcode Command Line Tools for Tauri desktop packaging

Install and verify:

```sh
make setup
make check
```

Run the backend:

```sh
make backend-dev
```

Run the frontend in another terminal:

```sh
make frontend-dev
```

Default local URLs:

- Backend: `http://127.0.0.1:3000`
- Frontend: `http://127.0.0.1:1420`
- Stremio manifest: `http://127.0.0.1:3000/manifest.json`

Useful checks:

```sh
curl http://127.0.0.1:3000/manifest.json
curl http://127.0.0.1:3000/stream/movie/tt0133093.json
curl http://127.0.0.1:3000/inspect/movie/tt0133093.json
```

## Configuration

Atlas reads optional Appwrite settings from environment variables:

```sh
APPWRITE_ENDPOINT=
APPWRITE_PROJECT_ID=
APPWRITE_API_KEY=
```

The backend binds to `127.0.0.1:3000` by default. Override it with:

```sh
ATLAS_BIND_ADDR=127.0.0.1:3001
```

Provider keys can be entered through the Settings page. The backend migrates legacy local secrets from `preferences.json` into Keychain when possible, then writes redacted preferences back to local/cloud storage.

## Stremio Install Flow

1. Start the backend with `make backend-dev`.
2. Open `http://127.0.0.1:3000/manifest.json` and confirm the manifest returns JSON.
3. In Stremio, install the local addon using `http://127.0.0.1:3000/manifest.json`.
4. Configure providers in the Atlas Settings page.
5. Use Smart Play or request streams through Stremio.

## Privacy and Operations

Runtime logs use structured `tracing` fields and can be filtered with `RUST_LOG`, for example:

```sh
RUST_LOG=backend=debug make backend-dev
```

Logs and telemetry avoid API keys, download URLs, magnets, torrent hashes, and raw Stremio playback identifiers. Local playback history stays in `backend/playback_history.json`, which is ignored by Git.

CI runs backend tests and frontend checks. Release packaging is handled by the Tauri release workflow on tags matching `v*`.
